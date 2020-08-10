use tracing_core::span::{Attributes, Id};
use tracing_core::{Dispatch, Subscriber};

use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use std::iter::Extend;
use std::{any::TypeId, marker::PhantomData, time::Instant};

// this function "remembers" the types of the subscriber and the formatter,
// so that we can downcast to something aware of them without knowing those
// types at the callsite.
pub(crate) struct WithContext {
    set: fn(&Dispatch, &Id, SpanRootTiming),
    take: fn(&Dispatch, &Id, f: &mut dyn FnMut(SpanTiming)),
}

impl WithContext {
    pub(crate) fn set<'a>(&self, dispatch: &'a Dispatch, id: &Id, value: SpanRootTiming) {
        (self.set)(dispatch, id, value)
    }

    pub(crate) fn take<'a>(&self, dispatch: &'a Dispatch, id: &Id, f: &mut dyn FnMut(SpanTiming)) {
        (self.take)(dispatch, id, f)
    }
}

/// The Tide tracing layer.
#[allow(missing_debug_implementations)]
pub struct TimingLayer<S> {
    ctx: WithContext,
    _subscriber: PhantomData<fn(S)>,
    _priv: (),
}

impl<S> TimingLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    /// Create a new instance.
    pub fn new() -> Self {
        let ctx = WithContext {
            set: Self::set_timing,
            take: Self::take_timing,
        };

        Self {
            ctx,
            _subscriber: PhantomData,
            _priv: (),
        }
    }

    fn set_timing(dispatch: &Dispatch, id: &Id, timing: SpanRootTiming) {
        let subscriber = dispatch
            .downcast_ref::<S>()
            .expect("subscriber should downcast to expected type; this is a bug!");
        let span = subscriber
            .span(id)
            .expect("registry should have a span for the current ID");

        let mut extensions = span.extensions_mut();
        extensions.insert(timing);
    }

    fn take_timing(dispatch: &Dispatch, id: &Id, f: &mut dyn FnMut(SpanTiming)) {
        let subscriber = dispatch
            .downcast_ref::<S>()
            .expect("subscriber should downcast to expected type; this is a bug!");
        let span = subscriber
            .span(id)
            .expect("registry should have a span for the current ID");

        let mut extensions = span.extensions_mut();
        if let Some(value) = extensions.remove::<SpanTiming>() {
            f(value);
        }
    }
}

impl<S> Layer<S> for TimingLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn new_span(&self, _attrs: &Attributes<'_>, id: &Id, cx: Context<'_, S>) {
        let span = cx.span(id).unwrap();
        let meta = span.metadata();
        let id = span.id();
        let timing = SpanTiming::new(id, meta.name(), meta.target());
        span.extensions_mut().insert(timing);
    }

    /// On exit fold the current span's timing into its parent timing.
    fn on_exit(&self, id: &Id, cx: Context<'_, S>) {
        let span = cx.span(id).unwrap();

        // Don't fold the root timing into its parent.
        if span.extensions().get::<SpanRootTiming>().is_some() {
            return;
        };

        // let name = span.metadata().name();
        let mut timing = match span.extensions_mut().remove::<SpanTiming>() {
            Some(timing) => timing,
            None => return,
        };

        // Snapshot the end time of the span.
        timing.end_timing();

        // fold the current timing into its parent's timing.
        let span = cx.span(id).unwrap();
        if let Some(parent_id) = span.parent_id() {
            let parent = cx.span(parent_id).expect("parent not found");
            if let Some(parent_timing) = parent.extensions_mut().get_mut::<SpanTiming>() {
                parent_timing.children.extend(timing.flatten());
            };
        }
    }

    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        match id {
            id if id == TypeId::of::<Self>() => Some(self as *const _ as *const ()),
            id if id == TypeId::of::<WithContext>() => Some(&self.ctx as *const _ as *const ()),
            _ => None,
        }
    }
}

/// Indicated the current struct is the root struct.
#[derive(Debug)]
pub struct SpanRootTiming;

/// A timing that represent a span beneath the root span.
#[derive(Debug)]
pub struct SpanTiming {
    pub(crate) target: &'static str,
    pub(crate) id: Id,
    pub(crate) span_name: &'static str,
    pub(crate) start_time: Instant,
    pub(crate) end_time: Option<Instant>,
    children: Vec<Self>,
}

// /// An empty type, denoting the current span is a root.
// struct RootSpanTiming {}

impl SpanTiming {
    fn new(id: Id, span_name: &'static str, target: &'static str) -> Self {
        Self {
            start_time: Instant::now(),
            end_time: None,
            id,
            span_name,
            target,
            children: vec![],
        }
    }

    fn end_timing(&mut self) {
        self.end_time = Some(Instant::now());
    }

    pub(crate) fn flatten(mut self) -> Vec<Self> {
        let mut children = vec![];
        std::mem::swap(&mut self.children, &mut children);
        children.push(self);
        children
    }
}
