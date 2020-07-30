use tide::{Error, Response, Result, StatusCode};
use tide_server_timing::Layer;
use tracing_core::Level;
use tracing_subscriber::layer::SubscriberExt;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    let sub = sub.with(Layer::new());
    tracing::subscriber::set_global_default(sub).expect("no global subscriber has been set");

    // records an event outside of any span context:
    tracing::info!("something happened");

    let span = tracing::info_span!("my_span");
    let _guard = span.enter();

    // records an event within "my_span".
    tracing::debug!("something happened inside my_span");
    Ok(())
}
