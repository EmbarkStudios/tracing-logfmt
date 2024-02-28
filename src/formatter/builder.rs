use tracing::Subscriber;
use tracing_subscriber::{
    fmt::{Layer, SubscriberBuilder},
    registry::LookupSpan,
};

use crate::{EventsFormatter, FieldsFormatter};

pub struct Builder {
    events: EventsFormatter,
    fields: FieldsFormatter,
}

/// Create a builder that can be used to configure the formatter.
///
/// Example:
/// ```rust
/// use tracing::dispatcher::{self, Dispatch};
/// use tracing_subscriber::Registry;
/// use tracing_subscriber::layer::SubscriberExt;
///
/// let subscriber = Registry::default()
///     .with(tracing_logfmt::builder().with_span_path(false).layer());
///
/// dispatcher::set_global_default(Dispatch::new(subscriber))
///     .expect("Global logger has already been set!");
/// ```
pub fn builder() -> Builder {
    Builder::new()
}

impl Builder {
    pub fn new() -> Self {
        Self {
            events: EventsFormatter::default(),
            fields: FieldsFormatter::default(),
        }
    }

    pub fn with_level(mut self, enable: bool) -> Self {
        self.events.with_level = enable;
        self
    }
    pub fn with_target(mut self, enable: bool) -> Self {
        self.events.with_target = enable;
        self
    }
    pub fn with_span_name(mut self, enable: bool) -> Self {
        self.events.with_span_name = enable;
        self
    }
    pub fn with_span_path(mut self, enable: bool) -> Self {
        self.events.with_span_path = enable;
        self
    }
    pub fn with_location(mut self, enable: bool) -> Self {
        self.events.with_location = enable;
        self
    }
    pub fn with_module_path(mut self, enable: bool) -> Self {
        self.events.with_module_path = enable;
        self
    }

    pub fn layer<S>(self) -> Layer<S, FieldsFormatter, EventsFormatter>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        tracing_subscriber::fmt::layer()
            .event_format(self.events)
            .fmt_fields(self.fields)
    }

    pub fn subscriber_builder(self) -> SubscriberBuilder<FieldsFormatter, EventsFormatter> {
        tracing_subscriber::fmt::Subscriber::builder()
            .event_format(self.events)
            .fmt_fields(self.fields)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
