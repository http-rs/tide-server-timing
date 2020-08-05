use crate::TimingLayer;
use tracing_subscriber::registry::{LookupSpan, Registry};

// bit of a gross hack; we're casting down to a registry when we really shouldn't assume one is present.
/// A trait ext to cast a Span down to a Registry.
pub(crate) trait SpanExt<T: 'static + Send + Sync> {
    /// Take an item from the Span's typemap.
    fn take_ext<F>(&self, f: F)
    where
        F: FnMut(Option<T>);
}

impl<T: 'static + Send + Sync> SpanExt<T> for tracing::Span {
    fn take_ext<F>(&self, mut f: F)
    where
        F: FnMut(Option<T>),
    {
        self.with_subscriber(|(id, subscriber)| {
            if subscriber.downcast_ref::<TimingLayer>().is_some() {
                if let Some(registry) = subscriber.downcast_ref::<Registry>() {
                    let span = registry
                        .span(id)
                        .expect("in new_span but span does not exist");
                    let mut extensions = span.extensions_mut();
                    let value = extensions.remove::<T>();
                    f(value);
                    // let (trace_id, parent_id) = get_xray_ctx(trace_id);
                    // if let Some(subsegment) = extensions.get_mut::<Subsegment>() {
                    //     subsegment.trace_id = trace_id;
                    //     if let Some(parent_id) = parent_id {
                    //         subsegment.parent_id = Some(parent_id);
                    //     }
                    // }
                }
            }
        });
    }
}

// impl<'a> SpanExt for &'a tracing::Span {
//     fn with_trace_id(self, trace_id: &str) -> Self {
//         self.with_subscriber(|(id, subscriber)| {
//             if subscriber.downcast_ref::<TimingLayer>().is_some() {
//                 if let Some(registry) = subscriber.downcast_ref::<Registry>() {
//                     let span = registry
//                         .span(id)
//                         .expect("in new_span but span does not exist");
//                     let mut extensions = span.extensions_mut();
//                     todo!()
//                     // let (trace_id, parent_id) = get_xray_ctx(trace_id);
//                     // if let Some(subsegment) = extensions.get_mut::<Subsegment>() {
//                     //     subsegment.trace_id = trace_id;
//                     //     if let Some(parent_id) = parent_id {
//                     //         subsegment.parent_id = Some(parent_id);
//                     //     }
//                     // }
//                 }
//             }
//         });
//         self
//     }
// }
