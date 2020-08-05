use tide_server_timing::TimingLayer;
use tracing_core::Level;
use tracing_subscriber::layer::SubscriberExt;

use async_std::task;
use std::time::Duration;
use tracing_futures::Instrument;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let (layer, _receiver) = TimingLayer::new();
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    let sub = sub.with(layer);
    tracing::subscriber::set_global_default(sub).expect("no global subscriber has been set");

    let mut app = tide::new();
    app.with(tide_server_timing::Timing::new());
    app.at("/").get(|_| async move { Ok("Hello chashu") });
    app.listen("localhost:8080").await?;
    Ok(())
}
