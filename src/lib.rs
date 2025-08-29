//! Formatter for logging `tracing_subscriber` events in logfmt format.
//!
//! Use as a formatting layer in tracing subscriber:
//! ```rust
//! use tracing::dispatcher::{self, Dispatch};
//! use tracing_subscriber::Registry;
//! use tracing_subscriber::layer::SubscriberExt;
//!
//! let subscriber = Registry::default()
//!     .with(tracing_logfmt::layer());
//!
//! dispatcher::set_global_default(Dispatch::new(subscriber))
//!     .expect("Global logger has already been set!");
//! ```

#![deny(unreachable_pub)]

mod formatter;
mod serializer;

pub use crate::formatter::builder::{builder, Builder};
pub use crate::formatter::{layer, EventsFormatter, FieldsFormatter};
pub use tracing_subscriber::fmt::format::FmtSpan;
