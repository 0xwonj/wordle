pub mod config;
pub mod types;
pub mod utils;

// Re-export common functionality
pub use config::Config;
pub use types::WordleResult;
pub use utils::{current_timestamp, init_logging, load_env};
