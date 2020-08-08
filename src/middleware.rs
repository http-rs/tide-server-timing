use tide::{Next, Request};
use tracing_futures::Instrument;

use http_types::trace::{Metric, ServerTiming};

use crate::span_ext::SpanExt;

/// Middleware that captures encodes all traces in a handler as `Server-Timing` headers.
#[derive(Debug)]
pub struct Timing {
    _priv: (),
}

impl Timing {
    /// Create a new instance of `Timing`.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Timing {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        // Create a fake span to guarantee we're always operating in a unique span.
        // TODO: We may not need this.
        let res = async move {
            // Mark the root span.
            let span = tracing::Span::current();
            span.insert_ext(crate::SpanRootTiming);

            // Run the current future to completion.
            let mut res = async move { next.run(req).await }
                .instrument(tracing::info_span!("tide endpoint handler"))
                .await;

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
        }
        .instrument(tracing::info_span!("tide-server-wrapper"))
        .await;
        Ok(res)
    }
}
