use hyper::{HeaderMap, Request};
use opentelemetry::propagation::Extractor;
use tower_http::trace::{DefaultMakeSpan, MakeSpan};
use tracing::Span;

#[derive(Clone, Debug)]
pub struct PropagatingSpan {
    inner: DefaultMakeSpan,
}

impl PropagatingSpan {
    pub fn new() -> Self {
        Self {
            inner: DefaultMakeSpan::new(),
        }
    }
}

impl<B> MakeSpan<B> for PropagatingSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&RequestHeaderCarrier::new(request.headers()))
        });
        let _guard = parent_context.attach();
        self.inner.make_span(request)
    }
}

struct RequestHeaderCarrier<'a> {
    headers: &'a HeaderMap,
}

impl<'a> RequestHeaderCarrier<'a> {
    fn new(headers: &'a HeaderMap) -> Self {
        RequestHeaderCarrier { headers }
    }
}

impl<'a> Extractor for RequestHeaderCarrier<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|h| h.as_str()).collect()
    }
}
