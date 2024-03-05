mod index;
mod serve_static;
mod echo;

use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::Method;
use hyper::{Request, Response};
use tracing::debug;

use crate::router::echo::echo;
use crate::router::echo::echo_upper;
use crate::router::echo::echo_reversed;
use crate::request_utils;
use crate::GlobalContext;

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

pub async fn router(
    req: Request<hyper::body::Incoming>,
    g_ctx: Arc<GlobalContext<'_>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let method = req.method();
    let path = req.uri().path();

    debug!("Received {} request at {}", method, path);
    // Serve static files separately
    if path.starts_with("/static") {
        return serve_static::get(req, g_ctx);
    }

    match (method, path) {
        (GET, "/healthcheck") => Ok(request_utils::send("OK")),
        (GET, "/") => index::get(req, g_ctx),
        (POST, "/echo") => echo(req),
        (POST, "/echo/uppercase") => echo_upper(req),
        (POST, "/echo/reversed") => echo_reversed(req).await,

        // Return 404 otherwise
        _ => Ok(request_utils::not_found())
    }
}

