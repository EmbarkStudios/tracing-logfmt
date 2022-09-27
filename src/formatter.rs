use std::fmt;

use tracing::field::Visit;
use tracing_core::{Event, Field, Subscriber};
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields};
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::serializer::{Serializer, SerializerError};

/// Creates a formatting layer
///
/// Example:
/// ```rust
/// use tracing::dispatcher::{self, Dispatch};
/// use tracing_subscriber::Registry;
/// use tracing_subscriber::layer::SubscriberExt;
///
/// let subscriber = Registry::default()
///     .with(tracing_logfmt::layer());
///
/// dispatcher::set_global_default(Dispatch::new(subscriber))
///     .expect("Global logger has already been set!");
/// ```
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

            let span = event
                .parent()
                .and_then(|id| ctx.span(id))
                .or_else(|| ctx.lookup_current());

            if let Some(span) = span {
                let span_path = span.scope().from_root().map(|span| span.name()).fold(
                    String::new(),
                    |mut acc, name| {
                        let add_separator = !acc.is_empty();

                        if add_separator {
                            acc.reserve(name.len() + 1);
                            acc.push('>');
                        } else {
                            acc.reserve(name.len());
                        }

                        acc.push_str(name);
                        acc
                    },
                );

                serializer.serialize_entry("span", span.name())?;
                serializer.serialize_entry("span_path", &span_path)?;
            }

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

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    use tracing::info_span;
    use tracing_subscriber::fmt::{MakeWriter, SubscriberBuilder};

    use super::*;

    #[derive(Clone, Debug)]
    struct MockWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    #[derive(Clone, Debug)]
    struct MockMakeWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    impl MockMakeWriter {
        fn new() -> Self {
            Self {
                buf: Arc::new(Mutex::new(Vec::new())),
            }
        }
        fn get_content(&self) -> String {
            let buf = self.buf.lock().unwrap();
            std::str::from_utf8(&buf[..]).unwrap().to_owned()
        }
    }

    impl MockWriter {
        fn new(buf: Arc<Mutex<Vec<u8>>>) -> Self {
            Self { buf }
        }
    }

    impl io::Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buf.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.buf.lock().unwrap().flush()
        }
    }

    impl<'a> MakeWriter<'a> for MockMakeWriter {
        type Writer = MockWriter;

        fn make_writer(&'a self) -> Self::Writer {
            MockWriter::new(self.buf.clone())
        }
    }

    fn subscriber() -> SubscriberBuilder<FieldsFormatter, EventsFormatter> {
        tracing_subscriber::fmt::Subscriber::builder()
            .event_format(EventsFormatter)
            .fmt_fields(FieldsFormatter)
    }

    #[test]
    fn test_span_and_span_path() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = subscriber().with_writer(mock_writer.clone()).finish();

        subscriber::with_default(subscriber, || {
            let _root = info_span!("root").entered();
            let _leaf = info_span!("leaf").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();

        assert!(content.contains("span=leaf"));
        assert!(content.contains("span_path=root>leaf"));
    }
}
