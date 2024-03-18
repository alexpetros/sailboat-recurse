mod index;
mod post;
mod debug;
mod serve_static;
mod healthcheck;
mod well_known;

use crate::router::well_known::webfinger;
use hyper::StatusCode;
use crate::server::context::Context;
use crate::server::response::ResponseResult;
use crate::sqlite::get_conn;
use std::sync::Arc;
use hyper::Method;
use tracing::debug;
use tracing::warn;
use tracing::error;

use crate::server::request::Request;
use crate::server::context::GlobalContext;
use crate::server::response;

pub const GET: &Method = &Method::GET;
pub const POST: &Method = &Method::POST;
pub const DELETE: &Method = &Method::DELETE;

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

    // Remove the query parameter for routing purposes
    let without_query = match path.split_once("?") {
        None => path,
        Some(x) => x.0
    };

    // Split into subroutes
    let subroutes: Vec<&str> = without_query.split("/").collect();

    let result = match (method, &subroutes[1..]) {
        (GET, [""]) => index::get(req, ctx),
        (GET, ["debug"]) => debug::get(req, ctx),

        (POST, ["post"]) => post::post(req, ctx).await,
        (DELETE, ["post", ..]) => post::delete(req, ctx),

        (GET, [".well_known", "webfinger"]) => webfinger::get(req, ctx).await,

        (GET, ["healthcheck"]) => healthcheck::get(req, ctx),
        _ => response::not_found(req, ctx)
    };

    if let Err(error) = result {
        if error.status_code == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{}", error);
        } else {
            warn!("{}", error);
        }
        response::send_status(error.status_code)
    } else {
        result
    }
}

