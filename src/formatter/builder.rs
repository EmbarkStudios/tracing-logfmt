use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

use crate::{EventsFormatter, FieldsFormatter};

pub struct Builder {
    events: EventsFormatter,
    fields: FieldsFormatter,
}

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

    pub fn build<S>(self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        tracing_subscriber::fmt::layer()
            .event_format(self.events)
            .fmt_fields(self.fields)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
