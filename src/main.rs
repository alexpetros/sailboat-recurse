use tokio_util::task::TaskTracker;
use crate::server::context::GlobalContext;
use std::env;
use std::sync::Arc;
use std::net::SocketAddr;
use hyper::body;
use minijinja::path_loader;
use tokio::signal;
use crate::config::Config;
use crate::router::router;
use server::request::Request;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{error, info, level_filters::LevelFilter};
use minijinja::Environment;

mod config;
mod router;
mod server;
mod static_files;
mod sqlite;
mod queries;
mod activitypub;


#[tokio::main]
async fn main() {
    let config = Config::new(env::args().collect());
    let port = config.port;
    let tracker = Arc::new(TaskTracker::new());

    // TODO this does not properly crash on startup if it can't bind a port
    tokio::task::spawn(run_server(port, tracker.clone()));

    // TODO upgrade this to handle interrupts
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Receieved shutdown signal, waiting for requests to end.");
            // TODO actually exit gracefully
            std::process::exit(0);
            // tracker.close();
            // tracker.wait().await;
        },
        Err(err) => { eprintln!("Unable to listen for shutdown signal: {}", err); },
    }

}

async fn run_server(port: u16, tracker: Arc<TaskTracker>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

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

    let g_ctx = Arc::new(GlobalContext::new(env, statics));

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Now listening at http://localhost:{}", port);

    loop {
        let (stream, _) = listener.accept().await?;

        // Wrapper to use Hyper traits with Tokio streams
        let io = TokioIo::new(stream);

        let shared_ctx = g_ctx.clone(); // Why is this necessary?
        let service = service_fn(move |req: hyper::Request<body::Incoming>| {
            router(Request(req), shared_ctx.clone())
        });

        // Spawn a tokio task to serve multiple connections concurrently
        tracker.spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                error!("Error serving connection: {}", err);
            }
        });
    }
}
