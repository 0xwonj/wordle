# Wordle Game Backend

A RESTful API backend for a Wordle game implemented in Rust using the Axum framework, with HTTP/2.0 support.

## Project Structure

The project has been refactored to use a library-based structure with multiple binary targets:

- **Library (`src/lib.rs`)**: Contains shared code and modules used by both binaries
- **Server Binary (`src/main.rs`)**: The main Wordle game server using Axum
- **CLI Binary (`src/bin/cli.rs`)**: Command-line client for interacting with the server

This approach follows Rust best practices by:
1. Reducing code duplication between binaries
2. Improving maintainability through better modularization
3. Making the codebase more testable
4. Allowing for future expansion with additional binaries

## Architecture

The application follows a layered architecture with the following components:

### API Layer
- **Routes**: Defined in `src/api/mod.rs`, handles HTTP request routing
- **Handlers**: Defined in `src/api/handlers.rs`, processes HTTP requests and returns responses
- **Models**: Data structures for API requests and responses

### Domain Layer
- **Game Logic**: Implemented in `src/game/mod.rs`, contains core game functionality
- **Models**: Domain models defined in `src/models.rs`
- **Word List**: A predefined list of words for the game

### Infrastructure Layer
- **Configuration**: Application settings in `src/config.rs`
- **Error Handling**: Custom error types in `src/error.rs`
- **Application State**: Shared state between requests in `src/state.rs`
- **Utilities**: Helper functions in `src/utils.rs`
- **HTTPS & HTTP/2.0**: TLS support with HTTP/2.0 for improved performance

## Features

- **Create Games**: Start a new Wordle game
- **Make Guesses**: Submit guesses for words, with validation and feedback
- **Track Game State**: Keep track of game progress, guesses, and results
- **Daily Words**: Like the original Wordle, all players get the same word on the same day
- **HTTP/2.0 Support**: Modern web protocol for faster loading and multiplexed connections
- **Robust Error Handling**: Clear error messages and codes for client feedback
- **Thread-safe State Management**: Safely manage game state across concurrent requests
- **Word Validation**: Validate guesses against a dictionary of allowed words

## API Endpoints

### Game Management
- `POST /api/game/new` - Create a new game
- `GET /api/game/{id}` - Get game status by ID
- `POST /api/game/{id}/guess` - Make a guess in a game

### Health Check
- `GET /api/health` - Health check endpoint

## Game Rules

1. The game selects a random 5-letter word for the player to guess
2. Each day, all players get the same word (just like the original Wordle)
3. Players have 6 attempts to guess the word correctly
4. After each guess, feedback is provided on the guess:
   - Correct letter in the correct position (Correct)
   - Correct letter in the wrong position (WrongPosition)
   - Letter not in the word (Wrong)
5. The game ends when the player guesses correctly or exhausts all attempts

## Data Flow

1. **Create Game**:
   - Server selects the daily word (based on current date)
   - A new game is created with unique ID
   - Game details are returned to the user

2. **Make Guess**:
   - User submits a guess for a specific game
   - Server validates the guess (correct length, valid word)
   - Server evaluates the guess against the secret word
   - Results are returned to the user
   - Game state is updated (attempts remaining, completed status)

3. **Get Game**:
   - User requests the current state of a game
   - Server retrieves the game by ID
   - Game details are returned to the user (the secret word is only revealed if the game is completed)

## Design Patterns

The application implements several design patterns for maintainability and scalability:

1. **Service Pattern**: Game logic is encapsulated in a `GameService` class
2. **Repository Pattern**: Game state is managed through a repository abstraction
3. **Builder Pattern**: Game and response objects are constructed using builder-like approaches
4. **Factory Pattern**: Game creation is handled through factory methods
5. **DTO Pattern**: Separate data transfer objects for API responses
6. **Singleton Pattern**: Daily word selection uses a cached, date-based approach

## Technical Decisions

### In-Memory Storage with Thread Safety
The current implementation uses in-memory storage with RwLock for thread safety. In a production environment, this should be replaced with a proper database.

### Error Handling
Custom error types with detailed error codes and messages for better client feedback.

### Concurrency
The application is designed to be thread-safe and can handle concurrent requests.

### Word Selection
Words are deterministically selected based on the current date, ensuring that all players get the same word on the same day, just like the original Wordle game.

## Development

### Prerequisites
- Rust 1.70+ (edition 2021)
- Cargo

### Running the Server
```bash
# Development
cargo run

# Release mode
cargo run --release
```

### Environment Variables
- `PORT` - Server port (default: 3000)

## Deployment

### Prerequisites

- Rust toolchain (installed via [rustup](https://rustup.rs/))
- Environment variables set or `.env` file (see Configuration section)

### Running the Server

```bash
# Clone the repository
git clone https://github.com/yourusername/wordle.git
cd wordle

# Generate self-signed TLS certificates for development
./generate_tls_cert.sh

# Build and run the application
cargo run
```

### HTTPS and HTTP/2.0 Configuration

The server supports HTTP/2.0 protocol for improved performance. HTTP/2.0 features include:

- **Multiplexing**: Multiple requests over a single connection
- **Header Compression**: Reduced overhead and improved efficiency 
- **Server Push**: Proactive resource delivery
- **Binary Protocol**: More efficient parsing and error detection

Configuration options can be set in the `.env` file:

```
# TLS Configuration
TLS_ENABLED=true               # Set to false to disable TLS (HTTP/2.0 requires TLS)
TLS_CERT_FILE=./keys/tls/certificate.pem  # Path to certificate file
TLS_KEY_FILE=./keys/tls/key.pem           # Path to private key file
```

For production deployment:
1. Obtain proper TLS certificates from a trusted Certificate Authority
2. Update the `.env` file with the paths to your certificates
3. Ensure your server has appropriate firewall rules for port 443

#### Performance Considerations

HTTP/2.0 provides significant performance benefits for API consumers, particularly for:
- Clients making multiple concurrent requests
- Mobile devices with limited bandwidth
- Applications with many small resources

## Future Improvements

- Add user authentication
- Implement persistent storage (PostgreSQL, Redis)
- Add timed game modes
- Create multiplayer functionality
- Implement leaderboards
- Add game analytics
- Support for different languages
- Custom word list support
- Daily challenges 