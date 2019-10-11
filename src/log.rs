pub use slog_env_cfg::Logger;

pub use slog::o;
pub use slog_scope::{logger, scope, with_logger};
pub use slog_scope_futures::FutureExt;

pub use slog_scope::{crit, debug, error, info, trace, warn};
pub use slog_scope::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
