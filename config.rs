/// Configuration constants for ttt

/// Maximum number of variables allowed in an expression
pub const MAX_VARIABLES: usize = 20;  // 2^20 = ~1M rows max

/// Maximum length allowed for variable names
pub const MAX_VARIABLE_NAME_LENGTH: usize = 50;

/// Maximum number of differences to show in equivalence check output
pub const MAX_DIFFERENCES_TO_SHOW: usize = 5;

/// Default timeout for complex operations (in seconds)
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// Application description
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
