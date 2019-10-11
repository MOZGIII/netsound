/// Base API we provide everywhere.
mod base {
    pub use slog::o;
    pub use slog_env_cfg::Logger;

    pub use slog_derive::{SerdeValue, KV};

    pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
}

/// Scopes-enabled API.
mod scopes {
    pub use super::base::*;

    pub use slog_scope::{logger, scope, with_logger};
    pub use slog_scope_futures::FutureExt;

    pub use slog_scope::{crit, debug, error, info, trace, warn};
}

// Scopes API is the default.
pub use scopes::*;

/// No-scopes API, for use where scopes are not available.
pub mod no_scopes {
    pub use super::base::*;

    pub use slog::{crit, debug, error, info, trace, warn};
}
