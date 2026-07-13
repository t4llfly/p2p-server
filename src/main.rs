use axum::{
    Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, broadcast};

#[derive(Clone)]
struct ClientInfo {
    session_id: u64,
    sender: broadcast::Sender<String>,
}

type RoomClients = HashMap<String, ClientInfo>;
type AppState = Arc<Mutex<HashMap<String, RoomClients>>>;

#[tokio::main]
async fn main() {
    let state = AppState::default();

    let app = Router::new()
        .route("/ws/:room", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    println!("Сервер запущен на порту 3030");

    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("Подключение к комнате: {}", room);
    ws.on_upgrade(move |socket| handle_socket(socket, room, state))
}

async fn handle_socket(socket: WebSocket, room: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, _) = broadcast::channel::<String>(100);

    let tx_for_recv = tx.clone();

    let client_name: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let client_name_clone = client_name.clone();

    let mut send_task = tokio::spawn(async move {
        let mut rx = tx.subscribe();
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let room_clone = room.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Some((ip_str, rest)) = text.split_once('|') {
                if let Some((name, session_id_str)) = rest.split_once('|') {
                    if let Ok(session_id) = session_id_str.parse::<u64>() {
                        let mut rooms = state_clone.lock().await;
                        let room_clients =
                            rooms.entry(room_clone.clone()).or_insert_with(HashMap::new);

                        if let Some(existing) = room_clients.get(name) {
                            if existing.session_id < session_id {
                                println!(
                                    "Удаляем старую сессию {} (session {})",
                                    name, existing.session_id
                                );
                                room_clients.remove(name);
                            } else {
                                println!(
                                    "Игнорируем старую сессию {} (session {})",
                                    name, session_id
                                );
                                continue;
                            }
                        }

                        println!("Добавляем клиента {} с session {}", name, session_id);
                        room_clients.insert(
                            name.to_string(),
                            ClientInfo {
                                session_id,
                                sender: tx_for_recv.clone(),
                            },
                        );

                        *client_name_clone.lock().await = Some(name.to_string());

                        for (other_name, client) in room_clients.iter() {
                            if other_name != name {
                                let _ = client.sender.send(text.clone());
                            }
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        },
    }

    if let Some(name) = client_name.lock().await.clone() {
        let mut rooms = state.lock().await;
        if let Some(room_clients) = rooms.get_mut(&room) {
            room_clients.remove(&name);
            println!("Удален клиент {} из комнаты {}", name, room);
        }
    }

    println!("Отключение от комнаты: {}", room);
}
