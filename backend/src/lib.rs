use http::Request;
use hyper::Body;
use jsonrpsee::{server::ServerBuilder, Methods};
use opentelemetry::trace::TracerProvider;
use opentelemetry_datadog::{ApiVersion, DatadogPropagator};
use opentelemetry_http::HeaderExtractor;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

mod dd;
use dd::{DatadogLogExporter, SetDatadogHeaderLayer};

// This function runs the JSON-RPC server with the specified methods and handles tracing and middleware.
pub async fn run_server(methods: impl Into<Methods>) -> anyhow::Result<()> {
    // Create a middleware layer for tracing HTTP requests
    let middleware = ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
        |request: &Request<Body>| {
            // Create an info span for the method call
            let span = info_span!("method.call");

            // Extract the parent context from incoming HTTP headers using the Datadog propagator
            let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
                propagator.extract(&HeaderExtractor(request.headers()))
            });

            // Set the parent context for the span
            span.set_parent(parent_cx);
            span
        },
    ));

    // Create the JSON-RPC server with the specified middleware
    let builder = ServerBuilder::new().set_middleware(middleware);
    let server = builder.build("0.0.0.0:8081".parse::<SocketAddr>()?).await?;

    // Start the JSON-RPC server with the provided methods and wait for it to stop
    let handle = server.start(methods)?;
    handle.stopped().await;
    Ok(())
}

// Initialize the OpenTelemetry tracer with Datadog exporters and set up the tracing subscriber.
pub fn init_tracer(endpoint: String, env: String, service: String, version: String) {
    // Create a Datadog log exporter to export OpenTelemetry span events to stdout.
    // The Datadog agent will collect these stdout logs.
    let datadog_log_exporter =
        DatadogLogExporter::new(env.clone(), service.clone(), version.clone());

    // Create a Datadog exporter to export OpenTelemetry spans to the Datadog agent.
    let datadog_exporter = opentelemetry_datadog::new_pipeline()
        .with_env(env)
        .with_service_name(service)
        .with_version(version)
        .with_agent_endpoint(endpoint)
        .with_api_version(ApiVersion::Version05)
        .build_exporter()
        .expect("Failed to build Datadog exporter");

    // Create a tracer provider that generates a tracer with both exporters.
    // When a span is created in the tracer, it will be exported with both exporters.
    let tracer_provider = opentelemetry::sdk::trace::TracerProvider::builder()
        .with_batch_exporter(datadog_exporter, opentelemetry::runtime::Tokio)
        .with_simple_exporter(datadog_log_exporter)
        .build();

    // Initialize a tracer for later use with a specific name and version.
    let tracer = tracer_provider.versioned_tracer("opentelemetry-datadog", Some("0.7.0"), None);

    // Set the tracer provider globally.
    opentelemetry::global::set_tracer_provider(tracer_provider);

    // Set the Datadog propagator for extracting and setting remote parent context from headers.
    opentelemetry::global::set_text_map_propagator(DatadogPropagator::new());

    // Filter tracing::Span based on the RUST_LOG environment variable.
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .expect("Failed to build EnvFilter from the environment variable 'RUST_LOG'");

    // Create a tracing subscriber registry and set up the OpenTelemetry layer for tracing spans.
    let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize the tracing subscriber with the specified filters and the OpenTelemetry layer.
    tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(opentelemetry_layer)
        .init();
}

pub fn shutdown_tracer() {
    opentelemetry::global::shutdown_tracer_provider();
}

pub fn get_client() -> tower_http::trace::Trace<
    dd::SetDatadogHeader<hyper::Client<hyper::client::HttpConnector>>,
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&Request<Body>) -> tracing::Span,
> {
    ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|_request: &Request<Body>| info_span!("client.request")),
        )
        .layer(SetDatadogHeaderLayer)
        .service(hyper::Client::new())
}
