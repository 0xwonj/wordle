# Wordle

A modern Wordle game backend implemented in Rust with the Axum framework, featuring HTTP/2 support, JWT authentication, and a CLI client.

## Project Overview

This project is a RESTful API that powers a Wordle game with the following features:

- **HTTP/2 Support**: Improved performance with multiplexing and header compression
- **JWT Authentication**: Secure user authentication with multiple signature algorithms (ed25519, RSA, HMAC)
- **CLI Client**: Interactive command-line client for playing the game
- **Core Game Logic**: Complete implementation of Wordle game rules
- **Clean Architecture**: Modular design with separation of concerns

## Architecture

The application follows a clean architecture pattern:

### Project Structure

```
src/
├── api/             # API routes, handlers, and error types
├── auth/            # Authentication logic (JWT, middleware, models)
├── bin/             # Binary targets (CLI client)
├── common/          # Common utilities and helper functions
├── core/            # Core application components and configurations
├── game/            # Core game logic and models
├── models/          # Shared data models
├── repository/      # Data access layer
├── server/          # Server configuration and setup
├── lib.rs           # Library exports
└── main.rs          # Server binary entry point
```

### Binary Targets

- **Server**: Main Wordle game server using Axum
- **CLI Client**: Command-line interface for interacting with the server

## Game Features

- **Daily Words**: All players get the same word on the same day
- **Game Persistence**: Games are saved and can be resumed
- **Robust Validation**: Comprehensive input validation and error handling
- **Guess Feedback**: Detailed feedback on letter positions (correct, wrong position, incorrect)
- **Statistics**: Game statistics and streaks

## API Endpoints

### Public Endpoints

- `GET /api/health` - Health check endpoint

### Protected Endpoints (Require Authentication)

- `POST /api/game/new` - Create a new game
- `GET /api/game/{id}` - Get game status by ID
- `POST /api/game/{id}/guess` - Make a guess in a game

## Security Features

- **JWT Authentication**: Secure token-based authentication
- **Multiple Signature Algorithms**: Support for ed25519, RSA, and HMAC
- **TLS Encryption**: HTTPS with TLS support
- **Password Hashing**: Secure password storage with bcrypt

## CLI Client

The CLI client provides an interactive terminal interface for:

- User authentication
- Creating new games
- Making guesses
- Viewing game statistics
- Colorful visualization of game results

## Getting Started

### Prerequisites

- Rust toolchain (2024 edition)
- OpenSSL development libraries

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/wordle.git
   cd wordle
   ```

2. Generate TLS certificates (for development):
   ```bash
   ./script/generate_tls_cert.sh
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

### Configuration

The application can be configured using environment variables or a `.env` file:

```
PORT=3000                            # Server port
LOG_LEVEL=info                       # Logging level

# JWT Authentication Settings
JWT_AUTH_TYPE=ed25519                # Options: "secret", "rsa", or "ed25519"
JWT_PUBLIC_KEY_FILE=./keys/jwt/public.pem
JWT_ISSUER=auth-service
JWT_AUDIENCE=wordle-service

# TLS Settings for HTTP/2.0 Support
TLS_ENABLED=true
TLS_CERT_FILE=./keys/tls/certificate.pem
TLS_KEY_FILE=./keys/tls/key.pem
```

### Running the Server

```bash
cargo run --release
```

### Using the CLI Client

```bash
cargo run --release --bin cli
```

## Development

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 
