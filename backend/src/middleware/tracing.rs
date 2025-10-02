// OpenTelemetry tracing middleware

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use opentelemetry::{
    trace::{SpanKind, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Middleware to extract W3C Traceparent and propagate OpenTelemetry context
pub async fn otel_trace_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Extract method and URI for span naming
    let method = request.method().clone();
    let uri = request.uri().clone();
    let span_name = format!("{} {}", method, uri.path());

    // Create new span with request metadata
    let span = tracing::info_span!(
        "http_request",
        otel.kind = ?SpanKind::Server,
        http.method = %method,
        http.target = %uri,
        http.scheme = "https",
    );

    // Extract traceparent header if present
    if let Some(traceparent) = request.headers().get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            tracing::debug!(traceparent = %traceparent_str, "Extracted traceparent");

            // Parse and inject into span context
            // The tracing-opentelemetry layer will automatically handle this
            span.set_parent(extract_trace_context(traceparent_str));
        }
    }

    // Execute request within span
    let response = {
        let _enter = span.enter();
        next.run(request).await
    };

    // Record response status
    let status_code = response.status();
    span.record("http.status_code", status_code.as_u16());

    // Set span status based on HTTP status code
    if status_code.is_server_error() {
        span.record("otel.status_code", "ERROR");
        span.record("otel.status_message", status_code.canonical_reason().unwrap_or("Unknown"));
    } else if status_code.is_client_error() {
        span.record("otel.status_code", "ERROR");
    } else {
        span.record("otel.status_code", "OK");
    }

    response
}

/// Extract OpenTelemetry context from traceparent header
fn extract_trace_context(traceparent: &str) -> Context {
    use opentelemetry::propagation::{TextMapPropagator, Extractor};
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use std::collections::HashMap;

    let mut carrier = HashMap::new();
    carrier.insert("traceparent".to_string(), traceparent.to_string());

    struct HashMapExtractor<'a>(&'a HashMap<String, String>);
    impl<'a> Extractor for HashMapExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).map(|s| s.as_str())
        }
        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|s| s.as_str()).collect()
        }
    }

    let propagator = TraceContextPropagator::new();
    propagator.extract(&HashMapExtractor(&carrier))
}

/// Inject current trace context into response headers
pub fn inject_trace_headers(response: &mut Response) {
    use opentelemetry::propagation::{TextMapPropagator, Injector};
    use opentelemetry_sdk::propagation::TraceContextPropagator;

    let span = tracing::Span::current();
    let context = span.context();

    struct HeaderInjector<'a>(&'a mut axum::http::HeaderMap);
    impl<'a> Injector for HeaderInjector<'a> {
        fn set(&mut self, key: &str, value: String) {
            if let Ok(header_value) = axum::http::HeaderValue::from_str(&value) {
                self.0.insert(key, header_value);
            }
        }
    }

    let propagator = TraceContextPropagator::new();
    propagator.inject_context(&context, &mut HeaderInjector(response.headers_mut()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_extraction() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let context = extract_trace_context(traceparent);

        // Context should be valid
        let span_context = context.span().span_context();
        assert!(span_context.is_valid());
    }

    #[test]
    fn test_trace_context_invalid() {
        let invalid_traceparent = "invalid";
        let context = extract_trace_context(invalid_traceparent);

        // Context should be invalid/empty
        let span_context = context.span().span_context();
        assert!(!span_context.is_valid());
    }
}
