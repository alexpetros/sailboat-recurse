use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{error, info, level_filters::LevelFilter};

use router::router;

mod router;

const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup logging (leaving at DEBUG level for now)
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let addr: SocketAddr = format!("127.0.0.1:{}", PORT).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Now listening at http://localhost:{}", PORT);

    loop {
        let (stream, _) = listener.accept().await?;

        // Wrapper to use Hyper traits with Tokio streams
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(router))
                .await
            {
                error!("Error serving connection: {}", err);
            }
        });
    }
}
