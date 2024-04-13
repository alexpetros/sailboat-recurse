mod serve_static;
mod healthcheck;
mod well_known;
mod index;
mod posts;
mod profiles;
mod debug;
mod search;
mod follow;
mod following;
mod switch;

use hyper::header::HOST;
use crate::server::error::ServerError;
use crate::router::well_known::webfinger;
use crate::server::server_response::ServerResponse;
use crate::sqlite::get_conn;
use std::sync::Arc;
use hyper::{Method, Request};
use hyper::body::Incoming;
use tracing::debug;
use tracing::warn;
use tracing::error;
use profiles::_profile_id;

use crate::server::server_request::ServerRequest;
use crate::server::context::GlobalContext;
use crate::server::server_response;

pub const GET: &Method = &Method::GET;
pub const POST: &Method = &Method::POST;
pub const DELETE: &Method = &Method::DELETE;

const DEFAULT_DB: &str = "./sailboat.db";

pub async fn router(req: Request<Incoming>, g_ctx: Arc<GlobalContext<'_>>) -> ServerResponse {
    let path = req.uri().path();
    let host = req.headers().get(HOST)
        .map(|h| h.to_str().unwrap_or("UNKNOWN"))
        .unwrap_or("UNKNOWN");

    if path != "/debug" {
        debug!("Received {} request at {} from host {}", &req.method(), path, host);
    }

    let db_path = std::env::var("DB_PATH").unwrap_or(DEFAULT_DB.to_owned());
    let db = get_conn(&db_path)?;

    let domain = if g_ctx.domain.is_some() {
        g_ctx.domain.clone().unwrap()
    } else {
        db.query_row("SELECT value FROM globals WHERE key = 'domain'", (), |row| { row.get(0) })?
    };

    let req = ServerRequest::new(req, &g_ctx, db, domain)?;

    // Serve static files separately
    if req.uri().path().starts_with("/static") {
        return serve_static::get(req).await;
    }

    // Remove the query parameter for routing purposes
    let without_query = match req.uri().path().split_once("?") {
        None => req.uri().path(),
        Some(x) => x.0
    };

    // Split into sub-routes
    let sub_routes: Vec<&str> = without_query.split("/").collect();

    match (req.method(), &sub_routes[1..]) {
        (GET, [""]) => index::get(req).await,
        (GET, ["debug"]) => debug::get(req),

        (POST, ["profiles"]) => profiles::post(req).await,
        (GET, ["profiles", "new"]) => profiles::new::get(req),
        (GET, ["profiles", _]) => _profile_id::get(req).await,

        (GET, ["switch", _]) => switch::get(req),

        (GET, ["search", ..]) => search::get(req),
        (POST, ["search", ..]) => search::post(req).await,

        (POST, ["posts"]) => posts::post(req).await,
        (DELETE, ["posts", ..]) => posts::delete(req),

        (POST, ["follow"]) => follow::post(req).await,
        (GET, ["following"]) => following::get(req).await,

        (GET, [".well-known", "webfinger"]) => webfinger::get(req).await,

        (GET, ["healthcheck"]) => healthcheck::get(req),
        _ => server_response::not_found(req)
    }
}

fn log_warn_and_send_specific_message(err: ServerError) -> ServerResponse {
    warn!("{}", err);
    server_response::send_status_and_message(err)
}

fn log_error_and_send_generic_message(err: ServerError) -> ServerResponse {
    error!("{}", err);
    server_response::send_status(err.status_code)
}

pub async fn serve(req: Request<Incoming>, g_ctx: Arc<GlobalContext<'_>>) -> ServerResponse {
    let result = router(req, g_ctx).await;
    if let Err(err) = result {
        // 4xx error messages are passed onto users, the rest aren't
        match err.status_code.as_u16() {
            400..=499 => log_warn_and_send_specific_message(err),
            500..=599 => log_error_and_send_generic_message(err),
            _ => log_error_and_send_generic_message(err)
        }
    } else {
        result
    }
}
