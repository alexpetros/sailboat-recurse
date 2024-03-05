use crate::config::Config;
use std::env;
use std::collections::HashMap;
use std::sync::Arc;
use hyper::body;
use hyper::Request;
use minijinja::path_loader;
use crate::router::router;
use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{error, info, level_filters::LevelFilter};
use minijinja::Environment;

mod config;
mod router;
mod request_utils;
mod static_files;


#[derive(Clone)]
pub struct Context<'a> {
    env: Arc<Environment<'a>>,
    statics: Arc<HashMap<String, Vec<u8>>>
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::new(env::args().collect());
    let port = config.port;
    run_server(port)
}

#[tokio::main]
async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup logging (leaving at DEBUG level for now)
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    // Setup template environment
    let mut env = Environment::new();
    env.set_loader(path_loader("src/templates"));
    let env = Arc::new(env);

    // Load static files
    let statics = static_files::load_static();
    let statics = Arc::new(statics);

    let ctx = Arc::new(Context { env, statics });

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Now listening at http://localhost:{}", port);

    loop {
        let (stream, _) = listener.accept().await?;

        // Wrapper to use Hyper traits with Tokio streams
        let io = TokioIo::new(stream);

        let shared_ctx = ctx.clone(); // Why is this necessary?
        let service = service_fn(move |req: Request<body::Incoming>| {
            router(req, shared_ctx.clone())
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
