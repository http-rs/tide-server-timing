use tide::{Next, Request};
use tracing_futures::Instrument;

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
            let fut = async move { Ok(next.run(req).await) };
            let span = tracing::info_span!("tide-server-timing");
            let res = Instrument::instrument(fut, span).await;

            // Now access the trace from the store.
            let span = tracing::span::Span::current();
            span.take_ext(|timings: Option<crate::SpanTiming>| {
                if let Some(timings) = timings {
                    let timings = timings.flatten();
                }
                // dbg!("hello", &res);
            });
            res
        }
        .instrument(tracing::trace_span!("tide-server-wrapper"))
        .await;
        res
    }
}
