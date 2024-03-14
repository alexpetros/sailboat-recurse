mod index;
mod post;
mod debug;
mod serve_static;
mod healthcheck;

use crate::server::context::Context;
use crate::server::response::ResponseResult;
use crate::sqlite::get_conn;
use std::sync::Arc;
use hyper::Method;
use tracing::debug;
use tracing::warn;

use crate::server::request::Request;
use crate::server::context::GlobalContext;
use crate::server::response;

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;
const DELETE: &Method = &Method::DELETE;

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
        (GET, "/healthcheck") => healthcheck::get(req, ctx),
        (GET, "/debug") => debug::get(req, ctx),
        (GET, "/") => index::get(req, ctx),
        (POST, "/post") => post::post(req, ctx).await,
        (DELETE, "/post") => post::post(req, ctx).await,

        // Return 404 if the request is not known
        _ => response::not_found(req, ctx).await
    };

    if let Err(error) = result {
        warn!("{}", error);
        response::server_error(g_ctx)
    } else {
        result
    }
}

