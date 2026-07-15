use axum::{
    Json, Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, broadcast};

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    success: bool,
    message: String,
    token: Option<String>,
    config_json: Option<String>,
}

#[derive(Deserialize)]
struct ConfigUpdateRequest {
    token: String,
    config_json: String,
}

struct ServerState {
    rooms: Mutex<HashMap<String, broadcast::Sender<String>>>,
    db: Mutex<Connection>,
}

type AppState = Arc<ServerState>;

#[tokio::main]
async fn main() {
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "p2p_voice.db".to_string());
    let db_conn = Connection::open(db_path).expect("Не удалось открыть базу данных");
    db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            token TEXT,
            config_json TEXT
        )",
            (),
        )
        .expect("Не удалось создать таблицу");

    let state = Arc::new(ServerState {
        rooms: Mutex::new(HashMap::new()),
        db: Mutex::new(db_conn),
    });

    let app = Router::new()
        .route("/ws/:room", get(ws_handler))
        .route("/api/register", post(api_register))
        .route("/api/login", post(api_login))
        .route("/api/config", post(api_update_config))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    println!("Сервер запущен на порту 3030");

    axum::serve(listener, app).await.unwrap();
}

async fn api_register(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Json<AuthResponse> {
    let username = payload.username.clone();
    let password = payload.password.clone();

    let hash =
        tokio::task::spawn_blocking(move || bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap())
            .await
            .unwrap();

    let db = state.db.lock().await;
    let result = db.execute(
        "INSERT INTO users (username, password_hash, config_json) VALUES (?1, ?2, '{}')",
        (&username, &hash),
    );

    match result {
        Ok(_) => Json(AuthResponse {
            success: true,
            message: "Регистрация успешна".into(),
            token: None,
            config_json: None,
        }),
        Err(_) => Json(AuthResponse {
            success: false,
            message: "Имя пользователя уже занято".into(),
            token: None,
            config_json: None,
        }),
    }
}

async fn api_login(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Json<AuthResponse> {
    let user_data = {
        let db = state.db.lock().await;
        let mut stmt = db
            .prepare("SELECT password_hash, config_json FROM users WHERE username = ?1")
            .unwrap();
        let mut rows = stmt.query([&payload.username]).unwrap();

        if let Ok(Some(row)) = rows.next() {
            let hash: String = row.get(0).unwrap();
            let config_json: String = row.get(1).unwrap();
            Some((hash, config_json))
        } else {
            None
        }
    };

    if let Some((hash, config_json)) = user_data {
        let password = payload.password.clone();

        let is_valid =
            tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash).unwrap_or(false))
                .await
                .unwrap();

        if is_valid {
            let token = uuid::Uuid::new_v4().to_string();

            let db = state.db.lock().await;
            db.execute(
                "UPDATE users SET token = ?1 WHERE username = ?2",
                (&token, &payload.username),
            )
            .unwrap();

            return Json(AuthResponse {
                success: true,
                message: "Успешный вход".into(),
                token: Some(token),
                config_json: Some(config_json),
            });
        }
    }

    Json(AuthResponse {
        success: false,
        message: "Неверный логин или пароль".into(),
        token: None,
        config_json: None,
    })
}

async fn api_update_config(
    State(state): State<AppState>,
    Json(payload): Json<ConfigUpdateRequest>,
) -> Json<AuthResponse> {
    let db = state.db.lock().await;

    let updated = db
        .execute(
            "UPDATE users SET config_json = ?1 WHERE token = ?2",
            (&payload.config_json, &payload.token),
        )
        .unwrap_or(0);

    if updated > 0 {
        Json(AuthResponse {
            success: true,
            message: "Настройки сохранены в облаке".into(),
            token: None,
            config_json: None,
        })
    } else {
        Json(AuthResponse {
            success: false,
            message: "Недействительный токен сессии".into(),
            token: None,
            config_json: None,
        })
    }
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
        let mut rooms = state.rooms.lock().await;
        rooms
            .entry(room.clone())
            .or_insert_with(|| broadcast::channel(16).0)
            .clone()
    };

    let mut rx = tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx_clone.send(text);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    println!("Отключение от комнаты: {}", room);
}
