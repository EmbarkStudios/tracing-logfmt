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
    crate::builder().layer()
}

/// A formatter that formats tracing-subscriber events into logfmt formatted log rows.
pub struct EventsFormatter {
    pub(crate) with_level: bool,
    pub(crate) with_target: bool,
    pub(crate) with_span_name: bool,
    pub(crate) with_span_path: bool,
    pub(crate) with_location: bool,
    pub(crate) with_module_path: bool,
    pub(crate) with_timestamp: bool,
    pub(crate) with_thread_names: bool,
    pub(crate) with_thread_ids: bool,
    #[cfg(feature = "ansi_logs")]
    pub(crate) with_ansi_color: bool,
}

impl Default for EventsFormatter {
    fn default() -> Self {
        Self {
            with_level: true,
            with_target: true,
            with_span_name: true,
            with_span_path: true,
            with_location: false,
            with_module_path: false,
            with_timestamp: true,
            with_thread_names: false,
            with_thread_ids: false,
            #[cfg(feature = "ansi_logs")]
            with_ansi_color: default_enable_ansi_color(),
        }
    }
}

#[cfg(feature = "ansi_logs")]
fn default_enable_ansi_color() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
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
        let mut serializer = Serializer::new(
            &mut writer,
            #[cfg(feature = "ansi_logs")]
            self.with_ansi_color,
        );

        let mut visit = || {
            let metadata = event.metadata();

            if self.with_timestamp {
                serializer.serialize_key("ts")?;
                serializer.writer.write_char('=')?;
                time::OffsetDateTime::now_utc()
                    .format_into(
                        &mut serializer,
                        &time::format_description::well_known::Rfc3339,
                    )
                    .map_err(|_e| fmt::Error)?;
            }

            if self.with_level {
                let level = match *metadata.level() {
                    tracing::Level::ERROR => "error",
                    tracing::Level::WARN => "warn",
                    tracing::Level::INFO => "info",
                    tracing::Level::DEBUG => "debug",
                    tracing::Level::TRACE => "trace",
                };

                #[cfg(feature = "ansi_logs")]
                {
                    if self.with_ansi_color {
                        let level_str = match *metadata.level() {
                            tracing::Level::ERROR => nu_ansi_term::Color::Red,
                            tracing::Level::WARN => nu_ansi_term::Color::Yellow,
                            tracing::Level::INFO => nu_ansi_term::Color::Green,
                            tracing::Level::DEBUG => nu_ansi_term::Color::Blue,
                            tracing::Level::TRACE => nu_ansi_term::Color::Purple,
                        }
                        .bold()
                        .paint(level);

                        serializer.serialize_entry("level", &level_str.to_string())?;
                    } else {
                        serializer.serialize_entry("level", level)?;
                    }
                }

                #[cfg(not(feature = "ansi_logs"))]
                serializer.serialize_entry("level", level)?;
            }

            if self.with_target {
                serializer.serialize_entry("target", metadata.target())?;
            }

            // Use same logic as tracing-subscriber for thread names and ids
            // https://github.com/tokio-rs/tracing/blob/efc690fa6bd1d9c3a57528b9bc8ac80504a7a6ed/tracing-subscriber/src/fmt/format/json.rs#L306
            if self.with_thread_names {
                let current_thread = std::thread::current();
                match current_thread.name() {
                    Some(name) => {
                        serializer.serialize_entry("thread.name", name)?;
                    }
                    // fall-back to thread id when name is absent and ids are not enabled
                    None if !self.with_thread_ids => {
                        serializer.serialize_entry(
                            "thread.name",
                            &format!("{:?}", current_thread.id()),
                        )?;
                    }
                    _ => {}
                }
            }

            if self.with_thread_ids {
                serializer.serialize_entry_no_quote("thread.id", std::thread::current().id())?;
            }

            let span = if self.with_span_name || self.with_span_path {
                event
                    .parent()
                    .and_then(|id| ctx.span(id))
                    .or_else(|| ctx.lookup_current())
            } else {
                None
            };

            if self.with_location {
                if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
                    serializer.serialize_entry("location", &format!("{}:{}", file, line))?;
                }
            }
            if self.with_module_path {
                if let Some(module) = metadata.module_path() {
                    serializer.serialize_entry("module_path", module)?;
                }
            }

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
                        let s = Serializer::new(
                            &mut span_path,
                            #[cfg(feature = "ansi_logs")]
                            self.with_ansi_color,
                        );
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
#[non_exhaustive]
pub struct FieldsFormatter {}

impl<'writer> FormatFields<'writer> for FieldsFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        mut writer: format::Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut serializer = Serializer::new(
            &mut writer,
            #[cfg(feature = "ansi_logs")]
            false,
        );
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
        builder::builder().subscriber_builder()
    }

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
    fn test_enable_thread_name_and_id() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_thread_names(true)
            .with_thread_ids(true)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        std::thread::Builder::new()
            .name("worker-1".to_string())
            .spawn(move || {
                subscriber::with_default(subscriber, || {
                    tracing::info!("message");
                });
            })
            .unwrap()
            .join()
            .unwrap();

        let content = mock_writer.get_content();
        println!("{:?}", content);
        assert!(content.contains("thread.name=worker-1"));
        assert!(content.contains("thread.id="));
    }

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
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
    #[cfg(not(feature = "ansi_logs"))]
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

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
    fn test_disable_span_and_span_path() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_span_name(false)
            .with_span_path(false)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            let _middle = info_span!("middle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();

        println!("{:?}", content);
        assert!(!content.contains("span="));
        assert!(!content.contains("span_path="));
        assert!(content.contains("level=info"));
        assert!(content.contains("ts=20"));
    }

    #[test]
    #[cfg(feature = "ansi_logs")]
    fn test_disable_ansi_color() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            // disable timestamp so it can be asserted
            .with_timestamp(false)
            .with_ansi_color(false)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        subscriber::with_default(subscriber, || {
            tracing::info!("message");
        });

        let content = mock_writer.get_content();

        // assert that there is no ansi color sequences
        assert_eq!(
            content,
            "level=info target=tracing_logfmt::formatter::tests message=message\n"
        );
    }

    #[test]
    #[cfg(feature = "ansi_logs")]
    fn test_enable_thread_name_and_id() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_thread_names(true)
            .with_thread_ids(true)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        std::thread::Builder::new()
            .name("worker-1".to_string())
            .spawn(move || {
                subscriber::with_default(subscriber, || {
                    tracing::info!("message");
                });
            })
            .unwrap()
            .join()
            .unwrap();

        let content = mock_writer.get_content();

        let thread_name_prefix = make_ansi_key_value("thread.name", "=");
        let thread_id_prefix = make_ansi_key_value("thread.id", "=");

        println!("{:?}", content);
        assert!(content.contains(&(thread_name_prefix + "worker-1")));
        assert!(content.contains(&thread_id_prefix));
    }

    #[test]
    #[cfg(feature = "ansi_logs")]
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

        let span = make_ansi_key_value("span", "=bottom");
        let span_path = make_ansi_key_value("span_path", "=\"top>mid dle>bottom\"");
        let ts = make_ansi_key_value("ts", "=20");

        println!("{:?}", content);
        assert!(content.contains(&span));
        assert!(content.contains(&span_path));
        assert!(content.contains("info"));
        assert!(content.contains(&ts));
    }
    #[test]
    #[cfg(feature = "ansi_logs")]
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

        let span = make_ansi_key_value("span", "=bottom");
        let span_path = make_ansi_key_value("span_path", "=top>middle>bottom");
        let ts = make_ansi_key_value("ts", "=20");

        println!("{}", content);
        assert!(content.contains(&span));
        assert!(content.contains(&span_path));
        assert!(content.contains("info"));
        assert!(content.contains(&ts));
    }

    #[test]
    #[cfg(feature = "ansi_logs")]
    fn test_disable_span_and_span_path() {
        use nu_ansi_term::Color;
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_span_name(false)
            .with_span_path(false)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            let _middle = info_span!("middle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();
        let message = make_ansi_key_value("message", "=message");
        let target = make_ansi_key_value("target", "=tracing_logfmt::formatter::tests");
        let ts = make_ansi_key_value("ts", "=");

        println!("{}", content);
        assert!(!content.contains("span="));
        assert!(!content.contains("span_path="));
        assert!(content.contains(&Color::Green.bold().paint("info").to_string()));
        assert!(content.contains(&ts));
        assert!(content.contains(&target));
        assert!(content.contains(&message));
    }

    #[test]
    #[cfg(all(not(feature = "ansi_logs"), windows))]
    fn test_enable_location() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_location(true)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            let _middle = info_span!("middle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();
        let split = content.split(r"location=src\formatter.rs:").last().unwrap();
        let line = &split[..3];
        assert!(line.parse::<u32>().is_ok());

        println!("{}", content);
        assert!(content.contains(r"location=src\formatter.rs:"));
        assert!(content.contains("info"));
    }

    #[test]
    #[cfg(all(not(feature = "ansi_logs"), not(windows)))]
    fn test_enable_location() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_location(true)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            let _middle = info_span!("middle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();
        let split = content.split("location=src/formatter.rs:").last().unwrap();
        let line = &split[..3];
        assert!(line.parse::<u32>().is_ok());

        println!("{}", content);
        assert!(content.contains("location=src/formatter.rs:"));
        assert!(content.contains("info"));
    }

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
    fn test_enable_module_path() {
        use tracing::subscriber;

        let mock_writer = MockMakeWriter::new();
        let subscriber = builder::builder()
            .with_module_path(true)
            .subscriber_builder()
            .with_writer(mock_writer.clone())
            .finish();

        subscriber::with_default(subscriber, || {
            let _top = info_span!("top").entered();
            let _middle = info_span!("middle").entered();
            let _bottom = info_span!("bottom").entered();

            tracing::info!("message");
        });

        let content = mock_writer.get_content();

        println!("{}", content);
        assert!(content.contains("module_path=tracing_logfmt::formatter::tests"));
        assert!(content.contains("info"));
    }

    #[cfg(feature = "ansi_logs")]
    fn make_ansi_key_value(key: &str, value: &str) -> String {
        use nu_ansi_term::Color;
        let mut key = Color::Rgb(109, 139, 140).bold().paint(key).to_string();
        key.push_str(value);
        key
    }
}
