use crate::{timing_layer::WithContext, SpanRootTiming, SpanTiming};

/// A trait ext to cast a Span down to a Registry.
pub(crate) trait SpanExt {
    /// Take an item from the Span's typemap.
    fn take_ext<F>(&self, f: F)
    where
        F: FnMut(SpanTiming);

    /// Insert a SpanRootTiming into the Span's typemap.
    fn insert_ext(&self, item: SpanRootTiming);
}

impl SpanExt for tracing::Span {
    fn take_ext<F>(&self, mut f: F)
    where
        F: FnMut(SpanTiming),
    {
        self.with_subscriber(|(id, subscriber)| {
            if let Some(ctx) = subscriber.downcast_ref::<WithContext>() {
                ctx.take(subscriber, id, &mut f)
            }
        });
    }

    fn insert_ext(&self, item: SpanRootTiming) {
        self.with_subscriber(|(id, subscriber)| {
            if let Some(ctx) = subscriber.downcast_ref::<WithContext>() {
                ctx.set(subscriber, id, item)
            }
        });
    }
}
