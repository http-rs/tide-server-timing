//! Server-Timing support for Tide + Tracing
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use tracing_core::span::{Attributes, Id};
use tracing_core::Subscriber;
use tracing_subscriber::layer::Context;

#[derive(Debug)]
pub struct Layer {
    _priv: (),
}

impl Layer {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for Layer {
    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, cx: Context<'_, S>) {
        dbg!(&attrs, id);
    }
}
