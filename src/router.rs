mod index;
mod debug;
mod serve_static;

use crate::request::ResponseResult;
use std::sync::Arc;
use hyper::Method;
use hyper::Request;
use tracing::debug;

use crate::request;
use crate::request::global_context::GlobalContext;

const GET: &Method = &Method::GET;
// const POST: &Method = &Method::POST;

pub async fn router(
    req: Request<hyper::body::Incoming>,
    g_ctx: Arc<GlobalContext<'_>>,
) -> ResponseResult {
    let method = req.method();
    let path = req.uri().path();

    if path != "/debug" {
        debug!("Received {} request at {}", method, path);
    }

    // Serve static files separately
    if path.starts_with("/static") {
        return serve_static::get(req, g_ctx);
    }

    let hander = match (method, path) {
        (GET, "/healthcheck") => request::ok,
        (GET, "/debug") => debug::get,
        (GET, "/") => index::get,

        // Return 404 if the request is not known
        _ => request::not_found
    };

    hander(req, g_ctx)
}

