use tracing_core::span::{Attributes, Id};
use tracing_core::Subscriber;

use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use async_channel::{bounded, Receiver, Sender};

use std::time::Instant;

/// The Tide tracing layer.
#[derive(Debug)]
pub struct TimingLayer {
    sender: Sender<SpanTiming>,
}

impl TimingLayer {
    /// Create a new instance.
    pub fn new() -> (Self, Receiver<SpanTiming>) {
        let (sender, receiver) = bounded(1);
        (Self { sender }, receiver)
    }
}

impl<S> Layer<S> for TimingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn new_span(&self, _attrs: &Attributes<'_>, id: &Id, cx: Context<'_, S>) {
        let span = cx.span(id).unwrap();
        let name = span.metadata().name();
        span.extensions_mut().insert(SpanTiming::new(name));
    }

    fn on_enter(&self, id: &Id, _cx: Context<'_, S>) {
        dbg!(id);
    }

    fn on_exit(&self, id: &Id, cx: Context<'_, S>) {
        let span = cx.span(id).unwrap();
        let name = span.metadata().name();
        let mut timing: SpanTiming = span
            .extensions_mut()
            .remove()
            .expect("timing on exit not found");

        // Finalize the timing.
        timing.end_timing();

        if name == "tide-server-timing" {
            self.sender.try_send(timing).expect("Could not send timing");
        } else {
            let span = cx.span(id).unwrap();
            if let Some(parent_id) = span.parent_id() {
                span.extensions_mut().insert(SpanTiming::new(name));
                let parent = cx.span(parent_id).expect("parent not found");
                if let Some(parent_timing) = parent.extensions_mut().get_mut::<SpanTiming>() {
                    parent_timing.children.push(timing);
                };
            }
        }
    }
}

/// A timing that represent a span beneath the root span.
#[derive(Debug)]
pub struct SpanTiming {
    name: &'static str,
    start_time: Instant,
    end_time: Option<Instant>,
    children: Vec<Self>,
}

// /// An empty type, denoting the current span is a root.
// struct RootSpanTiming {}

impl SpanTiming {
    fn new(name: &'static str) -> Self {
        Self {
            start_time: Instant::now(),
            end_time: None,
            name,
            children: vec![],
        }
    }

    fn end_timing(&mut self) {
        self.end_time = Some(Instant::now());
    }
}
