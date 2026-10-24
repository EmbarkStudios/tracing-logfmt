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
//!
//! ## `log` records
//!
//! With the `tracing-log` feature, which is enabled by default,
//! `tracing-logfmt` can support logs coming from the `log` crate.
//!
//! You must set your own [`LogTracer`]. Do it right after setting the
//! dispatcher:
//!
//! ```rust
//! # use tracing::dispatcher::{self, Dispatch};
//! # use tracing::level_filters::LevelFilter;
//! # use tracing_log::{AsLog, LogTracer};
//! # use tracing_subscriber::Registry;
//! # use tracing_subscriber::layer::SubscriberExt;
//!
//! let subscriber = Registry::default()
//!     .with(tracing_logfmt::layer());
//!
//! dispatcher::set_global_default(Dispatch::new(subscriber))
//!     .expect("Global logger has already been set!");
//!
//! LogTracer::builder()
//!     .with_max_level(LevelFilter::current().as_log())
//!     .init()
//!     .expect("a `log` logger has already been set");
//! ```
//!
//! [`LogTracer`]: https://docs.rs/tracing-log/0.2/tracing_log/struct.LogTracer.html

#![deny(unreachable_pub)]

mod formatter;
mod serializer;

pub use crate::formatter::builder::{builder, Builder};
pub use crate::formatter::{layer, EventsFormatter, FieldsFormatter};
pub use tracing_subscriber::fmt::format::FmtSpan;
