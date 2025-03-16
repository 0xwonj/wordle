use anyhow::Result;
use std::env;
use std::path::PathBuf;

/// JWT authentication configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// JWT signature verification method ("secret", "rsa", or "ed25519")
    pub auth_type: String,

    /// Public key value or file path
    pub public_key: String,

    /// Expected issuer
    pub issuer: String,

    /// Expected audience
    pub audience: String,
}

/// TLS configuration for HTTPS
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Whether to enable HTTPS
    pub enabled: bool,

    /// Path to the TLS certificate file
    pub cert_file: PathBuf,

    /// Path to the TLS key file
    pub key_file: PathBuf,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Whether to enable database persistence
    pub enabled: bool,

    /// Database connection URL
    pub url: String,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server port to listen on
    pub port: u16,

    /// JWT authentication settings
    pub jwt: JwtConfig,

    /// TLS configuration
    pub tls: TlsConfig,

    /// Database configuration
    pub database: DatabaseConfig,
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        // Get the port from environment or use default
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()?;

        // Load JWT settings
        let jwt_config = JwtConfig {
            auth_type: env::var("JWT_AUTH_TYPE").unwrap_or_else(|_| "ed25519".to_string()),
            public_key: if env::var("JWT_PUBLIC_KEY").is_ok() {
                env::var("JWT_PUBLIC_KEY").unwrap()
            } else if env::var("JWT_PUBLIC_KEY_FILE").is_ok() {
                let key_path = PathBuf::from(env::var("JWT_PUBLIC_KEY_FILE").unwrap());
                std::fs::read_to_string(key_path)?
            } else {
                return Err(anyhow::anyhow!(
                    "JWT_PUBLIC_KEY or JWT_PUBLIC_KEY_FILE must be set"
                ));
            },
            issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "auth-service".to_string()),
            audience: env::var("JWT_AUDIENCE").unwrap_or_else(|_| "wordle-service".to_string()),
        };

        // Load TLS settings
        let tls_enabled = env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()?;

        let tls_config = TlsConfig {
            enabled: tls_enabled,
            cert_file: PathBuf::from(
                env::var("TLS_CERT_FILE")
                    .unwrap_or_else(|_| "./keys/tls/certificate.pem".to_string()),
            ),
            key_file: PathBuf::from(
                env::var("TLS_KEY_FILE").unwrap_or_else(|_| "./keys/tls/key.pem".to_string()),
            ),
        };

        // Load database settings
        let db_enabled = env::var("DATABASE_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()?;

        let database_config = DatabaseConfig {
            enabled: db_enabled,
            url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/wordle".to_string()),
        };

        Ok(Self {
            port,
            jwt: jwt_config,
            tls: tls_config,
            database: database_config,
        })
    }
}
