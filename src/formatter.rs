//! Formatter for logging `tracing` events in the logfmt format.
//!
//! This module implements traits from `tracing_subscriber` to produce logfmt formatted logs.
//!
//! Use as a formatting layer in tracing subscriber:
//! ```rust
//! use tracing::dispatcher::{self, Dispatch};
//! use tracing_subscriber::Registry;
//! use tracing_subscriber::layer::SubscriberExt;
//!
//! let subscriber = Registry::default()
//!     .with(tracing_subscriber_logfmt::layer());
//!
//! dispatcher::set_global_default(Dispatch::new(subscriber))
//!     .expect("Global logger has already been set!");
//! ```
use std::fmt;

use tracing::field::Visit;
use tracing_core::{Event, Field, Subscriber};
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields};
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::serializer::{Serializer, SerializerError};

pub fn layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .event_format(EventsFormatter)
        .fmt_fields(FieldsFormatter)
}

/// A formatter that formats tracing-subscriber events into logfmt formatted log rows.
pub struct EventsFormatter;

impl<S, N> FormatEvent<S, N> for EventsFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut serializer = Serializer::new();
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|_e| fmt::Error)?;

        let mut visit = || {
            let metadata = event.metadata();

            serializer.serialize_entry("ts", &timestamp)?;
            serializer.serialize_entry("level", &metadata.level().as_str().to_lowercase())?;
            serializer.serialize_entry("target", metadata.target())?;

            let mut visitor = Visitor::new(&mut serializer);
            event.record(&mut visitor);
            visitor.state?;

            Ok(())
        };

        visit().map_err(|_e: SerializerError| fmt::Error)?;
        write!(writer, "{}", serializer.output)?;

        // Write all fields from spans
        if let Some(leaf_span) = ctx.lookup_current() {
            for span in leaf_span.scope().from_root() {
                let ext = span.extensions();
                let data = ext
                    .get::<FormattedFields<N>>()
                    .expect("Unable to find FormattedFields in extensions; this is a bug");

                if !data.is_empty() {
                    write!(writer, " ")?;
                    write!(writer, "{}", data)?;
                }
            }
        }

        writeln!(writer)
    }
}

/// A formatter that formats span fields into logfmt.
pub struct FieldsFormatter;

impl<'writer> FormatFields<'writer> for FieldsFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        mut writer: format::Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut serializer = Serializer::new();
        let mut visitor = Visitor::new(&mut serializer);
        fields.record(&mut visitor);
        write!(writer, "{}", serializer.output)
    }
}

struct Visitor<'a> {
    serializer: &'a mut Serializer,
    state: Result<(), SerializerError>,
}

impl<'a> Visitor<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        Self {
            serializer,
            state: Ok(()),
        }
    }
}

impl<'a> Visit for Visitor<'a> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        if self.state.is_ok() {
            self.record_debug(field, &value);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if self.state.is_ok() {
            self.record_debug(field, &value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if self.state.is_ok() {
            self.record_debug(field, &value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if self.state.is_ok() {
            self.record_debug(field, &value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if self.state.is_ok() {
            self.state = self.serializer.serialize_entry(field.name(), value);
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if self.state.is_ok() {
            self.record_debug(field, &format_args!("{}", value));
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.state.is_ok() {
            let val = format!("{:?}", value);
            self.record_str(field, &val);
        }
    }
}
