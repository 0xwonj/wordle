use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::Input;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Import from our library - only what we actually use
use wordle::{game::models::LetterResult as GameLetterResult, APP_VERSION};

/// CLI client for testing Wordle REST API server
#[derive(Parser, Debug)]
#[clap(author, version = APP_VERSION, about, long_about = None)]
struct Cli {
    /// API base URL
    #[clap(short, long, default_value = "https://localhost:3000")]
    api_url: String,

    /// Command to execute
    #[clap(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Simulate login to generate a test token
    Login {
        /// Username
        #[clap(short, long)]
        username: Option<String>,
    },

    /// Play a new game interactively
    Play {},

    /// Check health status of the server
    Health {},

    /// Start a new game
    New {},

    /// Check status of a game
    Status {
        /// Game ID
        #[clap(short, long)]
        game_id: Option<String>,
    },

    /// Make a guess in the current game
    Guess {
        /// The word to guess
        #[clap(short, long)]
        word: String,

        /// Game ID
        #[clap(short, long)]
        game_id: Option<String>,
    },
}

/// Configuration for storing settings and auth token
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    /// API base URL
    api_url: String,
    /// Authentication token
    token: Option<String>,
    /// User ID
    user_id: Option<String>,
    /// Username
    username: Option<String>,
    /// Current active game ID
    current_game_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "https://localhost:3000".to_string(),
            token: None,
            user_id: None,
            username: None,
            current_game_id: None,
        }
    }
}

impl Config {
    /// Loads the configuration from the config file
    ///
    /// # Returns
    ///
    /// A Result containing the loaded Config or an error
    fn load() -> Result<Self> {
        // Find config directory
        let mut config_path =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        config_path.push("wordle-cli");
        config_path.push("config.json");

        // Try to load config, or create default
        if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&config_str)?;
            Ok(config)
        } else {
            let config = Config::default();

            // Ensure parent directories exist
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Save default config
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

            Ok(config)
        }
    }

    /// Saves the configuration to the config file
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    fn save(&self) -> Result<()> {
        let mut config_path =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        config_path.push("wordle-cli");
        config_path.push("config.json");

        // Ensure parent directories exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save config
        std::fs::write(&config_path, serde_json::to_string_pretty(&self)?)?;

        Ok(())
    }
}

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// Username
    username: String,
    /// Issued at timestamp
    iat: usize,
    /// Expiration timestamp
    exp: usize,
    /// Issuer (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    iss: Option<String>,
    /// Audience (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<Vec<String>>,
    /// User roles (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<String>>,
    /// Email (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    /// Full name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

// API models based on server models
#[derive(Debug, Serialize, Deserialize)]
struct CreateGameRequest {}

#[derive(Debug, Serialize, Deserialize)]
struct GuessRequest {
    pub word: String,
}

// Define our local GuessResponse with proper derive attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuessResponse {
    pub word: String,
    pub results: Vec<GameLetterResult>,
}

// Define a local GameResponse that maps to the API version
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameResponse {
    pub id: String,
    pub attempts_remaining: u8,
    pub completed: bool,
    pub won: bool,
    pub word: Option<String>,
    pub guesses: Vec<GuessResponse>,
}

/// Display a single guess with color-coded results
///
/// # Arguments
///
/// * `guess` - The guess to display
fn display_guess(guess: &GuessResponse) {
    for (idx, letter_result) in guess.results.iter().enumerate() {
        let letter = guess.word.chars().nth(idx).unwrap_or('?');

        let colored_letter = match letter_result {
            GameLetterResult::Correct => letter.to_string().green().bold(),
            GameLetterResult::WrongPosition => letter.to_string().yellow().bold(),
            GameLetterResult::Wrong => letter.to_string().red(),
        };

        print!("{} ", colored_letter);
    }
    println!();
}

/// Display game information in a user-friendly format
///
/// # Arguments
///
/// * `game` - The game to display
fn display_game(game: &GameResponse) {
    println!("Game ID: {}", game.id);
    println!("Attempts remaining: {}", game.attempts_remaining);

    if !game.guesses.is_empty() {
        println!("\nGuesses so far:");
        for guess in &game.guesses {
            display_guess(guess);
            println!();
        }
    }

    if game.completed {
        if game.won {
            println!("{}", "Congratulations! You won!".green().bold());
        } else {
            println!("{}", "Game over! You lost.".red().bold());
        }

        if let Some(word) = &game.word {
            println!("The word was: {}", word.yellow().bold());
        }
    }
}

/// Generates a JWT token for testing using Ed25519 private key
///
/// # Arguments
///
/// * `username` - The username to associate with the token
///
/// # Returns
///
/// A Result containing the generated token and user ID tuple, or an error
fn generate_token(username: &str) -> Result<(String, String)> {
    // Generate a random user ID
    let user_id = Uuid::new_v4().to_string();

    // Current timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;

    // Create JWT claims aligned with server expectations
    let claims = Claims {
        sub: user_id.clone(),
        username: username.to_string(),
        iat: now,
        exp: now + 60 * 60 * 24,                       // 24 hours
        iss: Some("auth-service".to_string()),         // Match server configuration
        aud: Some(vec!["wordle-service".to_string()]), // Match server configuration
        roles: Some(vec!["user".to_string()]),
        email: None,
        name: None,
    };

    // Read private key from file
    let private_key = fs::read_to_string("keys/jwt/private.pem")
        .map_err(|e| anyhow::anyhow!("Failed to read private key: {}", e))?;

    // Create header with Ed25519 algorithm
    let header = Header::new(Algorithm::EdDSA);

    // Generate token
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_ed_pem(private_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Invalid Ed25519 key: {}", e))?,
    )?;

    Ok((token, user_id))
}

// API client for Wordle
struct WordleApi {
    client: Client,
    config: Config,
}

impl WordleApi {
    /// Creates a new WordleApi client with the specified API URL
    ///
    /// # Arguments
    ///
    /// * `api_url` - Optional API URL to override the one in config
    ///
    /// # Returns
    ///
    /// A Result containing the WordleApi instance or an error
    fn new(api_url: Option<String>) -> Result<Self> {
        // Load config
        let mut config = Config::load()?;

        // Override API URL if provided
        if let Some(url) = api_url {
            config.api_url = url;
            config.save()?;
        }

        // Create HTTP client with HTTP/2 support using rustls
        // Also disable certificate verification for testing purposes
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .use_rustls_tls() // Use rustls TLS implementation with HTTP/2 support
            .danger_accept_invalid_certs(true) // Accept self-signed certificates
            .build()?;

        Ok(Self { client, config })
    }

    /// Checks the health status of the API server
    ///
    /// # Returns
    ///
    /// A Result containing a boolean indicating if the server is healthy
    async fn health(&self) -> Result<bool> {
        let url = format!("{}/api/health", self.config.api_url);
        let resp = self.client.get(&url).send().await?;

        // Print response details for debugging
        println!(
            "Health check response: {} {}",
            resp.status(),
            resp.status().canonical_reason().unwrap_or("")
        );

        Ok(resp.status() == StatusCode::OK)
    }

    /// Generates JWT token and authenticates the user
    ///
    /// # Arguments
    ///
    /// * `username` - The username to use for authentication
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn login(&mut self, username: String) -> Result<()> {
        // Generate token using Ed25519 private key
        let (token, user_id) = generate_token(&username)?;

        // Update config
        self.config.token = Some(token);
        self.config.user_id = Some(user_id);
        self.config.username = Some(username);
        self.config.save()?;

        Ok(())
    }

    /// Creates a new game
    ///
    /// # Returns
    ///
    /// A Result containing the created game or an error
    async fn new_game(&mut self) -> Result<GameResponse> {
        self.ensure_auth()?;

        let url = format!("{}/api/game/new", self.config.api_url);
        let resp = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.token.as_ref().unwrap()),
            )
            .json(&CreateGameRequest {})
            .send()
            .await?;

        // Print response details for debugging
        println!(
            "New game response: {} {}",
            resp.status(),
            resp.status().canonical_reason().unwrap_or("")
        );

        if resp.status().is_success() {
            let game: GameResponse = resp.json().await?;

            // Update current game ID in config
            self.config.current_game_id = Some(game.id.clone());
            self.config.save()?;

            Ok(game)
        } else {
            let status = resp.status();
            let error_text = resp.text().await?;
            Err(anyhow::anyhow!(
                "Failed to create game: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Gets the status of a game
    ///
    /// # Arguments
    ///
    /// * `game_id` - Optional game ID, uses current game from config if None
    ///
    /// # Returns
    ///
    /// A Result containing the game status or an error
    async fn get_game(&self, game_id: Option<String>) -> Result<GameResponse> {
        self.ensure_auth()?;

        // Use provided game ID or current game from config
        let game_id = game_id
            .or_else(|| self.config.current_game_id.clone())
            .ok_or_else(|| anyhow::anyhow!("No game ID provided or saved in config"))?;

        let url = format!("{}/api/game/{}", self.config.api_url, game_id);
        let resp = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.token.as_ref().unwrap()),
            )
            .send()
            .await?;

        if resp.status().is_success() {
            let game: GameResponse = resp.json().await?;
            Ok(game)
        } else {
            let status = resp.status();
            let error_text = resp.text().await?;
            Err(anyhow::anyhow!(
                "Failed to get game: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Makes a guess in the current game
    ///
    /// # Arguments
    ///
    /// * `word` - The word to guess
    /// * `game_id` - Optional game ID, uses current game from config if None
    ///
    /// # Returns
    ///
    /// A Result containing the updated game or an error
    async fn make_guess(&self, word: String, game_id: Option<String>) -> Result<GameResponse> {
        self.ensure_auth()?;

        // Use provided game ID or current game from config
        let game_id = game_id
            .or_else(|| self.config.current_game_id.clone())
            .ok_or_else(|| anyhow::anyhow!("No game ID provided or saved in config"))?;

        let url = format!("{}/api/game/{}/guess", self.config.api_url, game_id);
        let resp = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.token.as_ref().unwrap()),
            )
            .json(&GuessRequest { word })
            .send()
            .await?;

        if resp.status().is_success() {
            let game: GameResponse = resp.json().await?;
            Ok(game)
        } else {
            let status = resp.status();
            let error_text = resp.text().await?;
            Err(anyhow::anyhow!(
                "Failed to make guess: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Ensures the user is authenticated with a valid token
    ///
    /// # Returns
    ///
    /// A Result indicating if the user is authenticated or an error
    fn ensure_auth(&self) -> Result<()> {
        if self.config.token.is_none() {
            Err(anyhow::anyhow!("Not authenticated. Please login first."))
        } else {
            Ok(())
        }
    }

    /// Plays an interactive game session with the user
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn play_interactive(&mut self) -> Result<()> {
        self.ensure_auth()?;

        println!("{}", "Starting a new Wordle game...".blue().bold());

        // Create a new game
        let mut spinner = Spinner::new(Spinners::Dots, "Creating new game...".into());
        let mut game = self.new_game().await?;
        spinner.stop();

        display_game(&game);

        // Main game loop
        while !game.completed {
            println!("\n{}", "Enter your guess (5 letters):".yellow());

            // Prompt for guess
            let guess: String = Input::new()
                .with_prompt(">")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.len() != 5 {
                        Err("Guess must be exactly 5 letters")
                    } else if !input.chars().all(|c| c.is_ascii_alphabetic()) {
                        Err("Guess must contain only letters")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;

            // Submit guess
            let mut spinner = Spinner::new(Spinners::Dots, "Submitting guess...".into());
            match self.make_guess(guess, None).await {
                Ok(updated_game) => {
                    spinner.stop();
                    game = updated_game;
                    display_game(&game);
                }
                Err(e) => {
                    spinner.stop();
                    println!("{}: {}", "Error".red().bold(), e);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if exists
    dotenv::dotenv().ok();

    // Parse command line arguments
    let cli = Cli::parse();

    // Create API client
    let mut api = WordleApi::new(Some(cli.api_url))?;

    // Execute requested command
    match cli.command {
        Commands::Login { username } => {
            // Get username from command line or prompt
            let username = match username {
                Some(u) => u,
                None => Input::<String>::new()
                    .with_prompt("Username")
                    .interact_text()?,
            };

            // Try to login
            println!("Generating auth token for {}...", username);
            match api.login(username).await {
                Ok(_) => println!("{}", "Token generated successfully!".green()),
                Err(e) => println!("{}: {}", "Token generation failed".red(), e),
            }
        }

        Commands::Health {} => {
            let mut spinner = Spinner::new(Spinners::Dots, "Checking server health...".into());
            match api.health().await {
                Ok(true) => {
                    spinner.stop();
                    println!("{}", "Server is healthy!".green());
                }
                Ok(false) => {
                    spinner.stop();
                    println!("{}", "Server is not healthy!".red());
                }
                Err(e) => {
                    spinner.stop();
                    println!("{}: {}", "Health check failed".red(), e);
                }
            }
        }

        Commands::New {} => {
            let mut spinner = Spinner::new(Spinners::Dots, "Creating new game...".into());
            match api.new_game().await {
                Ok(game) => {
                    spinner.stop();
                    println!("{}", "New game created!".green());
                    display_game(&game);
                }
                Err(e) => {
                    spinner.stop();
                    println!("{}: {}", "Failed to create new game".red(), e);
                }
            }
        }

        Commands::Status { game_id } => {
            let mut spinner = Spinner::new(Spinners::Dots, "Getting game status...".into());
            match api.get_game(game_id).await {
                Ok(game) => {
                    spinner.stop();
                    display_game(&game);
                }
                Err(e) => {
                    spinner.stop();
                    println!("{}: {}", "Failed to get game status".red(), e);
                }
            }
        }

        Commands::Guess { word, game_id } => {
            let mut spinner = Spinner::new(Spinners::Dots, "Submitting guess...".into());
            match api.make_guess(word, game_id).await {
                Ok(game) => {
                    spinner.stop();
                    display_game(&game);
                }
                Err(e) => {
                    spinner.stop();
                    println!("{}: {}", "Failed to submit guess".red(), e);
                }
            }
        }

        Commands::Play {} => match api.play_interactive().await {
            Ok(_) => println!("{}", "Thanks for playing!".green().bold()),
            Err(e) => println!("{}: {}", "Game error".red(), e),
        },
    }

    Ok(())
}
