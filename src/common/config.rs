use anyhow::{Context, Result};
use std::env;
use std::fs;
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
    /// Database connection URL
    pub url: String,
}

/// Main application configuration
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
        // Load port from PORT env var or use default
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;

        // Load JWT configuration
        let auth_type = env::var("JWT_AUTH_TYPE").unwrap_or_else(|_| "secret".to_string());
        let public_key = if auth_type == "secret" {
            env::var("JWT_SECRET").unwrap_or_default()
        } else {
            // Try to load from direct key value first
            match env::var("JWT_PUBLIC_KEY") {
                Ok(key) if !key.is_empty() => key,
                _ => {
                    // If direct key not provided, try to load from file
                    let key_file = env::var("JWT_PUBLIC_KEY_FILE")
                        .unwrap_or_else(|_| "./keys/jwt/public.pem".to_string());
                    fs::read_to_string(&key_file).with_context(|| {
                        format!("Failed to read JWT public key from {}", key_file)
                    })?
                }
            }
        };

        let jwt = JwtConfig {
            auth_type,
            public_key,
            issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "wordle".to_string()),
            audience: env::var("JWT_AUDIENCE").unwrap_or_else(|_| "users".to_string()),
        };

        // Load TLS configuration
        let tls = TlsConfig {
            enabled: env::var("TLS_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            cert_file: env::var("TLS_CERT_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("keys/cert.pem")),
            key_file: env::var("TLS_KEY_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("keys/key.pem")),
        };

        // Load database configuration
        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string()),
        };

        Ok(Self {
            port,
            jwt,
            tls,
            database,
        })
    }
}
