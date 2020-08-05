use tracing_core::span::{Attributes, Id};
use tracing_core::Subscriber;

use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use std::iter::Extend;
use std::time::Instant;

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
    fn new_span(&self, _attrs: &Attributes<'_>, id: &Id, cx: Context<'_, S>) {
        let span = cx.span(id).unwrap();
        let meta = span.metadata();
        let id = span.id();
        let timing = SpanTiming::new(id, meta.name(), meta.target());
        span.extensions_mut().insert(timing);
    }

    /// On exit fold the current span's timing into its parent timing.
    fn on_exit(&self, id: &Id, cx: Context<'_, S>) {
        let span = cx.span(id).unwrap();

        // Don't fold the root timing into its parent.
        if span.extensions().get::<SpanRootTiming>().is_some() {
            return;
        };

        // let name = span.metadata().name();
        let mut timing = match span.extensions_mut().remove::<SpanTiming>() {
            Some(timing) => timing,
            None => return,
        };

        // Snapshot the end time of the span.
        timing.end_timing();

        // fold the current timing into its parent's timing.
        let span = cx.span(id).unwrap();
        if let Some(parent_id) = span.parent_id() {
            let parent = cx.span(parent_id).expect("parent not found");
            if let Some(parent_timing) = parent.extensions_mut().get_mut::<SpanTiming>() {
                parent_timing.children.extend(timing.flatten());
            };
        }
    }
}

/// Indicated the current struct is the root struct.
#[derive(Debug)]
pub struct SpanRootTiming;

/// A timing that represent a span beneath the root span.
#[derive(Debug)]
pub struct SpanTiming {
    pub(crate) target: &'static str,
    pub(crate) id: Id,
    pub(crate) span_name: &'static str,
    pub(crate) start_time: Instant,
    pub(crate) end_time: Option<Instant>,
    children: Vec<Self>,
}

// /// An empty type, denoting the current span is a root.
// struct RootSpanTiming {}

impl SpanTiming {
    fn new(id: Id, span_name: &'static str, target: &'static str) -> Self {
        Self {
            start_time: Instant::now(),
            end_time: None,
            id,
            span_name,
            target,
            children: vec![],
        }
    }

    fn end_timing(&mut self) {
        self.end_time = Some(Instant::now());
    }

    pub(crate) fn flatten(mut self) -> Vec<Self> {
        let mut children = vec![];
        std::mem::swap(&mut self.children, &mut children);
        children.push(self);
        children
    }
}
