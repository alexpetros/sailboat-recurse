mod index;
mod debug;
mod serve_static;

use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::Method;
use hyper::{Request, Response};
use tracing::debug;

use crate::request_utils;
use crate::GlobalContext;

const GET: &Method = &Method::GET;
// const POST: &Method = &Method::POST;

pub async fn router(
    req: Request<hyper::body::Incoming>,
    g_ctx: Arc<GlobalContext<'_>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let method = req.method();
    let path = req.uri().path();

    if path != "/debug" {
        debug!("Received {} request at {}", method, path);
    }

    // Serve static files separately
    if path.starts_with("/static") {
        return serve_static::get(req, g_ctx);
    }

    match (method, path) {
        (GET, "/healthcheck") => Ok(request_utils::send("OK")),
        (GET, "/debug") => debug::get(req, g_ctx),
        (GET, "/") => index::get(req, g_ctx),

        // Return 404 otherwise
        _ => Ok(request_utils::not_found())
    }
}

