# P2P Voice Server

**Languages:** [English](README.md) | [Русский](README.ru.md) | [日本語](README.ja.md)

A peer-to-peer voice chat server built with Rust, featuring WebSocket communication, user authentication, and cloud configuration storage.

## Features

- **WebSocket-based real-time communication** - Connect to voice rooms via WebSocket
- **User authentication** - Register and login with secure password hashing (bcrypt)
- **Room system** - Create and join voice rooms dynamically
- **Cloud configuration** - Store and retrieve user settings
- **SQLite database** - Persistent storage for users and configurations

## Tech Stack

- **Rust** - Core language
- **Axum** - Web framework with WebSocket support
- **Tokio** - Async runtime
- **bcrypt** - Password hashing
- **rusqlite** - SQLite database bindings
- **serde/serde_json** - JSON serialization

## Installation

### Prerequisites

- Rust (edition 2024 or compatible)
- Cargo package manager

### Build and Run

```bash
# Clone the repository
git clone <repository-url>
cd p2p-server

# Build the project
cargo build --release

# Run the server
cargo run
```

The server will start on port `3030` by default.

## Configuration

Set the `DATABASE_URL` environment variable to specify the database path:

```bash
export DATABASE_URL=/path/to/p2p_voice.db
```

If not set, it defaults to `p2p_voice.db` in the current directory.

## API Endpoints

### Authentication

#### Register
```http
POST /api/register
Content-Type: application/json

{
    "username": "user123",
    "password": "securepassword"
}
```

#### Login
```http
POST /api/login
Content-Type: application/json

{
    "username": "user123",
    "password": "securepassword"
}
```

Response includes a session token and stored configuration.

### Configuration

#### Update Config
```http
POST /api/config
Content-Type: application/json

{
    "token": "session-token",
    "config_json": "{\"volume\": 80, \"mic_gain\": 50}"
}
```

### WebSocket

#### Connect to Room
```
WS /ws/:room
```

Connect to a voice room by replacing `:room` with your desired room name. Messages sent to the room are broadcast to all connected clients.

## Database Schema

The server creates a `users` table with the following structure:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    token TEXT,
    config_json TEXT
);
```

## Docker Support

Build and run using Docker Compose:

```bash
docker-compose up --build
```

## License

See the [LICENSE](LICENSE) file for details.

## Project Structure

```
p2p-server/
├── src/
│   └── main.rs      # Main application code
├── Cargo.toml       # Rust dependencies
├── Dockerfile       # Docker configuration
├── docker-compose.yml
└── README.md        # This file
```
