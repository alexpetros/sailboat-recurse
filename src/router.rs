mod index;
mod posts;
mod feeds;
mod debug;
mod serve_static;
mod healthcheck;
mod well_known;

use crate::router::well_known::webfinger;
use hyper::header::HOST;
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

const DEFAULT_DB: &str = "./sailboat.db";

pub async fn router(req: Request, g_ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let method = req.method();
    let path = req.uri().path();
    let origin = req.headers().get(HOST).ok_or("unknown");

    if path != "/debug" {
        debug!("Received {} request at {} from {:?}", method, path, origin);
    }

    let db_path = std::env::var("DB_PATH").unwrap_or(DEFAULT_DB.to_owned());
    let db = get_conn(&db_path)?;

    let ctx = Context::new(&g_ctx, db)?;

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

    match (method, &subroutes[1..]) {
        (GET, [""]) => index::get(req, ctx).await,
        (GET, ["debug"]) => debug::get(req, ctx),

        (GET, ["feeds", "new"]) => feeds::new::get(req, ctx),
        (GET, ["feeds", ..]) => feeds::get(req, ctx).await,
        (POST, ["feeds"]) => feeds::post(req, ctx).await,

        (POST, ["posts"]) => posts::post(req, ctx).await,
        (DELETE, ["posts", ..]) => posts::delete(req, ctx),

        (GET, [".well-known", "webfinger"]) => webfinger::get(req, ctx).await,

        (GET, ["healthcheck"]) => healthcheck::get(req, ctx),
        _ => response::not_found(req, ctx)
    }

}

pub async fn serve(req: Request, g_ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let result = router(req, g_ctx).await;
    if let Err(error) = result {
        if error.status_code == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{}", error);
            response::send_status(error.status_code)
        } else {
            warn!("{}", error);
            response::send_status_and_message(error)
        }
    } else {
        result
    }
}
