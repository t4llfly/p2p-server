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

type AppState = Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>;

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

    let tx = {
        let mut rooms = state.lock().await;
        rooms
            .entry(room.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };

    let mut rx = tx.subscribe();
    let tx_clone = tx.clone();

    let room_for_send = room.clone();
    let room_for_recv = room.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            println!("[{}] Отправляю клиенту: {}", room_for_send, msg);
            if sender.send(Message::Text(msg)).await.is_err() {
                println!("❌ [{}] Ошибка отправки клиенту", room_for_send);
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            println!("[{}] Получено от клиента: {}", room_for_recv, text);
            match tx_clone.send(text.clone()) {
                Ok(count) => println!("[{}] Транслирую {} подписчикам", room_for_recv, count),
                Err(e) => println!("❌ [{}] Ошибка трансляции: {}", room_for_recv, e),
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            println!("[{}] send_task завершился", room);
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            println!("[{}] recv_task завершился", room);
            send_task.abort();
        },
    }

    println!("Отключение от комнаты: {}", room);
}
