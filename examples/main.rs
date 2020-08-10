use tide_server_timing::{TimingLayer, TimingMiddleware};
use tracing_core::Level;
use tracing_subscriber::layer::SubscriberExt;

use async_std::task;
use std::time::Duration;
use tracing_futures::Instrument;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish()
        .with(TimingLayer::new());

    tracing::subscriber::set_global_default(sub).expect("no global subscriber has been set");

    let mut app = tide::new();
    app.with(TimingMiddleware::new());
    app.at("/").get(|_| async move {
        async move {
            task::sleep(Duration::from_millis(10)).await;
            Ok("Hello chashu")
        }
        .instrument(tracing::info_span!("my cool span"))
        .await
    });
    app.listen("localhost:8080").await?;
    Ok(())
}
