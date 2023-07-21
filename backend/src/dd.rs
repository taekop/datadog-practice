use http::{Request, Response};
use opentelemetry::sdk::export::trace::SpanExporter;
use opentelemetry_http::HeaderInjector;
use tower::Service;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Debug)]
pub struct DatadogLogExporter {
    env: String,
    service: String,
    version: String,
}

impl DatadogLogExporter {
    pub fn new(env: String, service: String, version: String) -> Self {
        Self {
            env,
            service,
            version,
        }
    }
}

impl SpanExporter for DatadogLogExporter {
    fn export(
        &mut self,
        batch: Vec<opentelemetry::sdk::export::trace::SpanData>,
    ) -> futures::future::BoxFuture<'static, opentelemetry::sdk::export::trace::ExportResult> {
        for span in batch {
            let trace_id = u128::from_be_bytes(span.span_context.trace_id().to_bytes());
            let span_id = u64::from_be_bytes(span.span_context.span_id().to_bytes());
            for event in span.events {
                let log = event.name;
                let mut level = "INFO".to_owned();
                let mut code_namespace = "unknown".to_owned();
                let mut code_filepath = "unknown".to_owned();
                let mut code_lineno = "0".to_owned();
                for attr in event.attributes {
                    if attr.key == opentelemetry::Key::from_static_str("level") {
                        level = attr.value.to_string();
                    } else if attr.key == opentelemetry::Key::from_static_str("code.namespace") {
                        code_namespace = attr.value.to_string();
                    } else if attr.key == opentelemetry::Key::from_static_str("code.filepath") {
                        code_filepath = attr.value.to_string();
                    } else if attr.key == opentelemetry::Key::from_static_str("code.lineno") {
                        code_lineno = attr.value.to_string();
                    }
                }

                let datetime = chrono::DateTime::<chrono::Utc>::from(event.timestamp)
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, false);

                println!("{} {} [dd.env={} dd.service={} dd.version={} dd.trace_id={} dd.span_id={}] {}:{}:{} - {}", datetime, level, self.env, self.service, self.version, trace_id, span_id, code_namespace, code_filepath, code_lineno, log);
            }
        }
        Box::pin(async { Ok(()) })
    }
}

pub struct SetDatadogHeaderLayer;

impl<S> tower::Layer<S> for SetDatadogHeaderLayer {
    type Service = SetDatadogHeader<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SetDatadogHeader { inner }
    }
}

pub struct SetDatadogHeader<S> {
    inner: S,
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for SetDatadogHeader<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let cx = tracing::Span::current().context();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut()))
        });
        self.inner.call(req)
    }
}
