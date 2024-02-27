use std::sync::Arc;
use hyper::body;
use hyper::Request;
use crate::router::router;
use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{error, info, level_filters::LevelFilter};
use minijinja::{Environment};

mod router;
mod request_utils;

const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup logging (leaving at DEBUG level for now)
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    // Setup template environment
    let mut env = Environment::new();
    env.add_template("hello", "<!DOCTYPE html>\n<title>Hello World</title>\n<h1>Hello {{ name }}!</h1>").unwrap();
    let env = Arc::new(env);

    let addr: SocketAddr = format!("127.0.0.1:{}", PORT).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Now listening at http://localhost:{}", PORT);

    loop {
        let (stream, _) = listener.accept().await?;

        // Wrapper to use Hyper traits with Tokio streams
        let io = TokioIo::new(stream);

        let shared_env = env.clone(); // Why is this necessary?
        let service = service_fn(move |req: Request<body::Incoming>| {
            router(req, shared_env.clone())
        });

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                error!("Error serving connection: {}", err);
            }
        });
    }
}
