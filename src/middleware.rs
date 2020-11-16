use tide::{Next, Request};
use tracing::Span;
use tracing_futures::Instrument;

use http_types::trace::{Metric, ServerTiming};

use crate::span_ext::SpanExt;

/// Middleware that captures encodes all traces in a handler as `Server-Timing` headers.
#[derive(Debug)]
pub struct TimingMiddleware {
    _priv: (),
}

impl TimingMiddleware {
    /// Create a new instance of `Timing`.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for TimingMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        // Create a fake span to guarantee we're always operating in a unique span.
        // TODO: We may not need this.
        let fut = async move {
            // Mark the root span.
            let span = tracing::Span::current();
            span.insert_ext(crate::SpanRootTiming);

            let curr = Span::current();
            let values = curr.attributes.values();

            // Run the current future to completion.
            let fut = async move { next.run(req).await };
            let span = tracing::info_span!("tide endpoint handler");
            let mut res = fut.instrument(span).await;

            // Now access the trace from the store.
            let span = tracing::span::Span::current();
            span.take_ext(|timings: crate::SpanTiming| {
                let raw_timings = timings.flatten();
                let mut timings = ServerTiming::new();

                for timing in raw_timings {
                    let dur = match timing.end_time {
                        Some(end_time) => end_time.duration_since(timing.start_time),
                        None => continue, // This would be the active span, which we ignore.
                    };

                    let name = timing.id.into_u64().to_string();
                    let desc = format!("{} ({})", timing.span_name, timing.target);

                    let metric = Metric::new(name, Some(dur), Some(desc))
                        .expect("Invalid metric formatting");
                    timings.push(metric);
                }
                timings.apply(&mut res);
            });
            res
        };

        let span = tracing::info_span!("tide-server-wrapper");
        let res = fut.instrument(span).await;
        Ok(res)
    }
}
