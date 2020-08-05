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
        let res = async move {
            // Run the current future to completion.
            let fut = async move { next.run(req).await };
            let span = tracing::info_span!("tide_endpoint");
            let mut res = Instrument::instrument(fut, span).await;

            // Now access the trace from the store.
            let span = tracing::span::Span::current();
            span.take_ext(|timings: Option<crate::SpanTiming>| {
                if let Some(timings) = timings {
                    let raw_timings = timings.flatten();
                    let mut timings = ServerTiming::new();

                    for timing in raw_timings {
                        let name = timing.name;
                        let dur = match timing.end_time {
                            Some(end_time) => end_time.duration_since(timing.start_time),
                            None => continue, // This would be the active span, which we ignore.
                        };

                        let metric = Metric::new(name.to_owned(), Some(dur), None)
                            .expect("Invalid metric formatting");
                        timings.push(metric);
                    }
                    timings.apply(&mut res);
                }
            });
            res
        }
        .instrument(tracing::trace_span!("tide-server-wrapper"))
        .await;
        Ok(res)
    }
}
