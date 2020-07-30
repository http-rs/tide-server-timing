use tracing_core::span::{Attributes, Id};
use tracing_core::Subscriber;
use tracing_subscriber::layer::Context;

#[derive(Debug)]
pub(crate) struct Layer {
    _priv: (),
}

impl Layer {
    pub(crate) fn new() -> Self {
        Self { _priv: () }
    }
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for Layer {
    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, _cx: Context<'_, S>) {
        dbg!(&attrs, id);
    }
}
