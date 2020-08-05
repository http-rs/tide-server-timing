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

mod middleware;
mod span_ext;
mod timing_layer;

pub use middleware::Timing;
pub use timing_layer::{SpanRootTiming, SpanTiming, TimingLayer};
