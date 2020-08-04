use tide_server_timing::TimingLayer;
use tracing_core::Level;
use tracing_subscriber::layer::SubscriberExt;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    let sub = sub.with(TimingLayer::new());
    tracing::subscriber::set_global_default(sub).expect("no global subscriber has been set");

    let mut app = tide::new();
    app.with(tide_server_timing::Timing::new());
    app.at("/").get(|_| async move {
        tracing::info!("something happened");
        let span = tracing::info_span!("my_span");
        let _guard = span.enter();
        tracing::debug!("something happened inside my_span");
        let res = Ok("Hello chashu");
        res
    });
    app.listen("localhost:8080").await?;
    Ok(())
}
