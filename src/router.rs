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

    let subroutes: Vec<&str> = path.split("/").collect();

    let result = match (method, &subroutes[1..]) {
        (GET, []) => index::get(req, ctx),
        (GET, ["debug"]) => debug::get(req, ctx),

        (POST, ["post"]) => post::post(req, ctx).await,
        (DELETE, ["post"]) => post::delete(req, ctx),

        (GET, ["healthcheck"]) => healthcheck::get(req, ctx),
        _ => response::not_found(req, ctx)
    };

    // let result = match sub_route {
    //     Some("") => index::get(req, ctx),
    //     Some("post") => post::router(req, ctx).await,
    //     Some("debug") => debug::router(req, ctx),
    //     Some("healthcheck") => healthcheck::router(req, ctx),
    //
    //     // Return 404 if the request is not known
    //     _ => response::not_found(req, ctx)
    // };

    if let Err(error) = result {
        error!("{}", error);
        response::server_error(g_ctx)
    } else {
        result
    }
}

