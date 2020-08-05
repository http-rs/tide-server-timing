use crate::TimingLayer;
use tracing_subscriber::registry::{LookupSpan, Registry};

// bit of a gross hack; we're casting down to a registry when we really shouldn't assume one is present.
/// A trait ext to cast a Span down to a Registry.
pub(crate) trait SpanExt {
    /// Take an item from the Span's typemap.
    fn take_ext<F, T>(&self, f: F)
    where
        F: FnMut(T),
        T: 'static + Send + Sync;

    /// Insert an item into the Span's typemap.
    fn insert_ext<T>(&self, item: T)
    where
        T: 'static + Send + Sync;
}

impl SpanExt for tracing::Span {
    fn take_ext<F, T: 'static + Send + Sync>(&self, mut f: F)
    where
        F: FnMut(T),
    {
        self.with_subscriber(|(id, subscriber)| {
            if subscriber.downcast_ref::<TimingLayer>().is_some() {
                let registry = subscriber
                    .downcast_ref::<Registry>()
                    .expect("Expected a tracing-subscriber tracer with a Registry");

                let span = registry
                    .span(&id)
                    .expect("in new_span but span does not exist");

                let mut extensions = span.extensions_mut();
                if let Some(value) = extensions.remove::<T>() {
                    f(value);
                }
            }
        });
    }

    fn insert_ext<T>(&self, item: T)
    where
        T: 'static + Send + Sync,
    {
        self.with_subscriber(|(id, subscriber)| {
            if subscriber.downcast_ref::<TimingLayer>().is_some() {
                let registry = subscriber
                    .downcast_ref::<Registry>()
                    .expect("Expected a tracing-subscriber tracer with a Registry");

                let span = registry
                    .span(&id)
                    .expect("in new_span but span does not exist");

                let mut extensions = span.extensions_mut();
                extensions.insert(item);
            }
        });
    }
}
