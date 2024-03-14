mod index;
mod post;
mod debug;
mod serve_static;

use crate::request::global_context::Context;
use crate::request::ResponseResult;
use crate::sqlite::get_conn;
use std::sync::Arc;
use hyper::Method;
use tracing::debug;
use tracing::warn;

use crate::request;
use crate::request::Request;
use crate::request::global_context::GlobalContext;

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

pub async fn router(req: Request, g_ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let method = req.method();
    let path = req.uri().path();

    if path != "/debug" {
        debug!("Received {} request at {}", method, path);
    }

    let db = get_conn("./sailboat.db")?;
    let ctx = Context { global: g_ctx.clone(), db };

    // Serve static files separately
    if path.starts_with("/static") {
        return serve_static::get(req, ctx).await;
    }

    let result = match (method, path) {
        (GET, "/healthcheck") => request::ok(),
        (GET, "/debug") => debug::get(req, ctx),
        (GET, "/") => index::get(req, ctx),
        (POST, "/post") => post::post(req, ctx).await,

        // Return 404 if the request is not known
        _ => request::not_found(req, ctx).await
    };

    if let Err(error) = result {
        warn!("{}", error);
        request::server_error(g_ctx)
    } else {
        result
    }
}

