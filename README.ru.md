# VVcall Server

**Языки:** [English](README.md) | [Русский](README.ru.md) | [日本語](README.ja.md)

Сервер голосового чата peer-to-peer, написанный на Rust, с поддержкой WebSocket, аутентификацией пользователей и облачным хранением конфигураций.

## Возможности

- **Обмен данными в реальном времени через WebSocket** - Подключение к голосовым комнатам через WebSocket
- **Аутентификация пользователей** - Регистрация и вход с безопасным хешированием паролей (bcrypt)
- **Система комнат** - Создание и присоединение к голосовым комнатам динамически
- **Облачная конфигурация** - Хранение и загрузка пользовательских настроек
- **База данных SQLite** - Постоянное хранение пользователей и конфигураций

## Технологический стек

- **Rust** - Основной язык
- **Axum** - Веб-фреймворк с поддержкой WebSocket
- **Tokio** - Асинхронная среда выполнения
- **bcrypt** - Хеширование паролей
- **rusqlite** - Привязки к базе данных SQLite
- **serde/serde_json** - Сериализация JSON

## Установка

### Требования

- Rust (edition 2024 или совместимая версия)
- Менеджер пакетов Cargo

### Сборка и запуск

```bash
# Клонировать репозиторий
git clone https://github.com/vvcall-dev/server.git
cd server

# Собрать проект
cargo build --release

# Запустить сервер
cargo run
```

Сервер запустится на порту `3030` по умолчанию.

## Конфигурация

Установите переменную окружения `DATABASE_URL` для указания пути к базе данных:

```bash
export DATABASE_URL=/path/to/vvcall.db
```

Если не установлено, по умолчанию используется `vvcall.db` в текущей директории.

## API Endpoints

### Аутентификация

#### Регистрация
```http
POST /api/register
Content-Type: application/json

{
    "username": "user123",
    "password": "securepassword"
}
```

#### Вход
```http
POST /api/login
Content-Type: application/json

{
    "username": "user123",
    "password": "securepassword"
}
```

Ответ включает токен сессии и сохранённую конфигурацию.

### Конфигурация

#### Обновление конфигурации
```http
POST /api/config
Content-Type: application/json

{
    "token": "session-token",
    "config_json": "{\"volume\": 80, \"mic_gain\": 50}"
}
```

### WebSocket

#### Подключение к комнате
```
WS /ws/:room
```

Подключитесь к голосовой комнате, заменив `:room` на желаемое имя комнаты. Сообщения, отправленные в комнату, транслируются всем подключённым клиентам.

## Схема базы данных

Сервер создаёт таблицу `users` со следующей структурой:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    token TEXT,
    config_json TEXT
);
```

## Поддержка Docker

Сборка и запуск через Docker Compose:

```bash
docker-compose up --build
```

## Лицензия

Подробности см. в файле [LICENSE](LICENSE).

## Структура проекта

```
p2p-server/
├── src/
│   └── main.rs      # Основной код приложения
├── Cargo.toml       # Зависимости Rust
├── Dockerfile       # Конфигурация Docker
├── docker-compose.yml
└── README.ru.md     # Этот файл
```
