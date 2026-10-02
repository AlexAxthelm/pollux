pub const SKIP_FORWARD_SECS: u32 = 30;
pub const SKIP_BACKWARD_SECS: u32 = 15;
pub const RESUME_REWIND_SECS: u32 = 3;
pub const REFRESH_INTERVAL_HOURS: u32 = 12;
/// Backoff applied to a 429 that carries no usable `Retry-After`.
pub const RATE_LIMIT_BACKOFF_SECS: i64 = 3600;
/// How long auto-refresh leaves a feed alone after a failed refresh, so a broken feed
/// isn't retried on every foreground. Manual refresh ignores it.
pub const FAILURE_BACKOFF_SECS: i64 = 900;
