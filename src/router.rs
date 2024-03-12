mod index;
mod debug;
mod serve_static;

use crate::request::global_context::Context;
use crate::request::ResponseResult;
use crate::sqlite::get_conn;
use std::sync::Arc;
use hyper::Method;
use hyper::Request;
use hyper::body::Incoming;
use tracing::debug;
use tracing::warn;

use crate::request;
use crate::request::global_context::GlobalContext;

const GET: &Method = &Method::GET;
// const POST: &Method = &Method::POST;

pub async fn router(req: Request<Incoming>, g_ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let method = req.method();
    let path = req.uri().path();

    if path != "/debug" {
        debug!("Received {} request at {}", method, path);
    }

    let db = get_conn("./sailboat.db")?;
    let ctx = Context { global: g_ctx.clone(), db };

    // Serve static files separately
    if path.starts_with("/static") {
        return serve_static::get(req, ctx);
    }

    let hander = match (method, path) {
        (GET, "/healthcheck") => request::ok,
        (GET, "/debug") => debug::get,
        (GET, "/") => index::get,

        // Return 404 if the request is not known
        _ => request::not_found
    };

    let result = hander(req, ctx);
    if let Err(error) = result {
        warn!("{}", error);
        request::server_error(g_ctx.clone())
    } else {
        result
    }
}

