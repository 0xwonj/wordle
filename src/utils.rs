// Helper macro for RwLock read operations with type annotations
#[macro_export]
macro_rules! with_read_lock {
    ($lock:expr, $err_msg:expr, $body:expr) => {
        match $lock.read() {
            Ok(guard) => $body(&guard),
            Err(e) => {
                tracing::error!("{}: {}", $err_msg, e);
                None
            }
        }
    };
}

// Helper macro for RwLock write operations with type annotations
#[macro_export]
macro_rules! with_write_lock {
    ($lock:expr, $err_msg:expr, $default:expr, $body:expr) => {
        match $lock.write() {
            Ok(mut guard) => $body(&mut guard),
            Err(e) => {
                tracing::error!("{}: {}", $err_msg, e);
                $default
            }
        }
    };
}

/// Returns the current time as an RFC3339 formatted string
pub fn current_time_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Truncates a string to a maximum length, appending "..." if truncated
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
