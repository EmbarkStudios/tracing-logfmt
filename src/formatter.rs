pub(crate) mod builder;

use std::fmt::{self, Write};

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
    crate::builder().build()
}

/// A formatter that formats tracing-subscriber events into logfmt formatted log rows.
pub struct EventsFormatter {
    pub(crate) with_level: bool,
    pub(crate) with_target: bool,
    pub(crate) with_span_name: bool,
    pub(crate) with_span_path: bool,
}

impl Default for EventsFormatter {
    fn default() -> Self {
        Self {
            with_level: true,
            with_target: true,
            with_span_name: true,
            with_span_path: true,
        }
    }
}

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
        let mut serializer = Serializer::new(&mut writer);

        let mut visit = || {
            let metadata = event.metadata();

            serializer.serialize_key("ts")?;
            serializer.writer.write_char('=')?;
            time::OffsetDateTime::now_utc()
                .format_into(
                    &mut serializer,
                    &time::format_description::well_known::Rfc3339,
                )
                .map_err(|_e| fmt::Error)?;

            if self.with_level {
                let level = match *metadata.level() {
                    tracing::Level::ERROR => "error",
                    tracing::Level::WARN => "warn",
                    tracing::Level::INFO => "info",
                    tracing::Level::DEBUG => "debug",
                    tracing::Level::TRACE => "trace",
                };
                serializer.serialize_entry("level", level)?;
            }

            if self.with_target {
                serializer.serialize_entry("target", metadata.target())?;
            }

            let span = if self.with_span_name || self.with_span_path {
                event
                    .parent()
                    .and_then(|id| ctx.span(id))
                    .or_else(|| ctx.lookup_current())
            } else {
                None
            };

            if let Some(span) = span {
                if self.with_span_name {
                    serializer.serialize_entry("span", span.name())?;
                }

                if self.with_span_path {
                    serializer.serialize_key("span_path")?;
                    serializer.writer.write_char('=')?;

                    let needs_quote = span
                        .scope()
                        .from_root()
                        .any(|span| span.name().chars().any(crate::serializer::need_quote));

                    // if none of the span names need to be quoted we can do things a bit faster
                    if needs_quote {
                        let mut required_capacity = 0;
                        let mut insert_sep = false;
                        for span in span.scope().from_root() {
                            if insert_sep {
                                required_capacity += 1;
                            }
                            required_capacity += span.name().len();
                            insert_sep = true;
                        }

                        let mut span_path = String::with_capacity(required_capacity);
                        let s = Serializer::new(&mut span_path);
                        let mut insert_sep = false;
                        for span in span.scope().from_root() {
                            if insert_sep {
                                s.writer.write_char('>')?;
                            }
                            s.writer.write_str(span.name())?;
                            insert_sep = true;
                        }
                        serializer.serialize_value(&span_path)?;
                    } else {
                        let mut insert_sep = false;
                        for span in span.scope().from_root() {
                            if insert_sep {
                                serializer.writer.write_char('>')?;
                            }
                            serializer.writer.write_str(span.name())?;
                            insert_sep = true;
                        }
                    }
                }
            }

            let mut visitor = Visitor::new(&mut serializer);
            event.record(&mut visitor);
            visitor.state?;

            Ok(())
        };

        visit().map_err(|_e: SerializerError| fmt::Error)?;

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
#[derive(Default)]
pub struct FieldsFormatter {}

impl<'writer> FormatFields<'writer> for FieldsFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        mut writer: format::Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut serializer = Serializer::new(&mut writer);
        let mut visitor = Visitor::new(&mut serializer);
        fields.record(&mut visitor);
        Ok(())
    }
}

struct Visitor<'a, W> {
    serializer: &'a mut Serializer<W>,
    state: Result<(), SerializerError>,
    debug_fmt_buffer: String,
}

impl<'a, W> Visitor<'a, W> {
    fn new(serializer: &'a mut Serializer<W>) -> Self {
        Self {
            serializer,
            state: Ok(()),
            debug_fmt_buffer: String::new(),
        }
    }
}

impl<'a, W> Visit for Visitor<'a, W>
where
    W: fmt::Write,
{
    fn record_f64(&mut self, field: &Field, value: f64) {
        if self.state.is_ok() {
            self.record_debug_no_quote(field, value);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if self.state.is_ok() {
            self.record_debug_no_quote(field, value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if self.state.is_ok() {
            self.record_debug_no_quote(field, value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if self.state.is_ok() {
            self.record_debug_no_quote(field, value);
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
            self.debug_fmt_buffer.clear();
            let _ = write!(self.debug_fmt_buffer, "{:?}", value);
            self.state = self
                .serializer
                .serialize_entry(field.name(), &self.debug_fmt_buffer);
        }
    }
}

impl<'a, W> Visitor<'a, W>
where
    W: fmt::Write,
{
    fn record_debug_no_quote(&mut self, field: &Field, value: impl fmt::Debug) {
        if self.state.is_ok() {
            self.state = self
                .serializer
                .serialize_entry_no_quote(field.name(), value);
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
            .event_format(EventsFormatter::default())
            .fmt_fields(FieldsFormatter::default())
    }

    #[test]
    fn test_span_and_span_path_with_quoting() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = subscriber().with_writer(mock_writer.clone()).finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            // the ' ' requires quoting
            let _middle = info_span!("mid dle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();

        println!("{:?}", content);
        assert!(content.contains("span=bottom"));
        assert!(content.contains("span_path=\"top>mid dle>bottom\""));
        assert!(content.contains("info"));
        assert!(content.contains("ts=20"));
    }

    #[test]
    fn test_span_and_span_path_without_quoting() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = subscriber().with_writer(mock_writer.clone()).finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            let _middle = info_span!("middle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();

        println!("{:?}", content);
        assert!(content.contains("span=bottom"));
        assert!(content.contains("span_path=top>middle>bottom"));
        assert!(content.contains("info"));
        assert!(content.contains("ts=20"));
    }
}
