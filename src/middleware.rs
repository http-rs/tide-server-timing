use tide::{Next, Request};
use tracing_futures::Instrument;
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
        let fut = async move { Ok(next.run(req).await) };
        let span = tracing::info_span!("tide-server-timing");
        Instrument::instrument(fut, span).await
    }
}
