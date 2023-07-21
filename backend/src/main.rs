use async_trait::async_trait;
use http::Request;
use hyper::Body;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use tower::Service;
use tracing::{info, instrument};

use backend::{get_client, init_tracer, run_server, shutdown_tracer};

#[rpc(server)]
pub trait Rpc {
    #[method(name = "increment")]
    async fn increment(&self, val: u64) -> Result<u64, ErrorObjectOwned>;
}

#[derive(Debug)]
pub struct RpcServerImpl;

#[async_trait]
impl RpcServer for RpcServerImpl {
    #[instrument(name = "increment")]
    async fn increment(&self, val: u64) -> Result<u64, ErrorObjectOwned> {
        let x = get_value().await;
        info!("Got value {x}");
        let y = x + val;
        set_value(y).await;
        info!("Set value {y}");
        Ok(y)
    }
}

async fn get_value() -> u64 {
    let mut client = get_client();
    let response = client
        .call(
            Request::builder()
                .uri("http://db:8082")
                .method("POST")
                .body(Body::from(
                    "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"get\",\"params\":[]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let x = serde_json::from_slice::<serde_json::Value>(&body)
        .unwrap()
        .get("result")
        .unwrap()
        .as_u64()
        .unwrap();
    return x;
}

async fn set_value(x: u64) {
    let mut client = get_client();
    client
        .call(
            Request::builder()
                .uri("http://db:8082")
                .method("POST")
                .body(Body::from(format!(
                    "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"set\",\"params\":[{x}]}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = "http://datadog-agent:8126".to_owned();
    let env = std::env::var("DD_ENV").expect("environment variable not found: DD_ENV");
    let service = std::env::var("DD_SERVICE").expect("environment variable not found: DD_SERVICE");
    let version = std::env::var("DD_VERSION").expect("environment variable not found: DD_VERSION");
    init_tracer(endpoint, env, service, version);
    run_server(RpcServerImpl.into_rpc()).await?;
    shutdown_tracer();
    Ok(())
}
