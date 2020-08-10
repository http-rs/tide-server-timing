use tide_server_timing::{TimingLayer, TimingMiddleware};
use tracing_core::Level;
use tracing_subscriber::layer::SubscriberExt;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish()
        .with(TimingLayer::new());

    tracing::subscriber::set_global_default(sub).expect("no global subscriber has been set");

    let mut app = tide::new();
    app.with(TimingMiddleware::new());
    app.at("/").get(|_| async move { Ok("Hello chashu") });
    app.listen("localhost:8080").await?;
    Ok(())
}
