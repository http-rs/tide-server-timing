use tracing_core::span::{Attributes, Id};
use tracing_core::Subscriber;

use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use std::time::Duration;

/// The Tide tracing layer.
#[derive(Debug)]
pub struct TimingLayer {
    _priv: (),
}

impl TimingLayer {
    /// Create a new instance.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl<S> Layer<S> for TimingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, _cx: Context<'_, S>) {
        dbg!(&attrs, id);
    }

    fn on_enter(&self, id: &Id, _cx: Context<'_, S>) {
        dbg!(id);
    }

    fn on_exit(&self, id: &Id, cx: Context<'_, S>) {
        // if cx.ext() has data {}
        let span = cx.span(id).unwrap();
        let parent = span.parent_id().unwrap();
        dbg!("closing", id);
    }
}

/// A timing that represent the root span.
#[derive(Debug)]
pub struct RootSpanTiming {}

/// A timing that represent a span beneath the root span.
#[derive(Debug)]
pub struct SpanTiming {
    name: String,
    dur: Option<Duration>,
    nested: Vec<SpanTiming>,
}
