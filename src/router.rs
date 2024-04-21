mod debug;
mod feeds;
mod follow;
mod following;
mod healthcheck;
mod index;
mod login;
mod logout;
mod posts;
mod profiles;
mod search;
mod serve_static;
mod switch;
mod well_known;

use crate::router::well_known::webfinger;
use crate::server::error::ServerError;
use crate::server::server_response::ServerResponse;
use crate::sqlite::get_conn;
use feeds::_feed_handle;
use hyper::body::Incoming;
use hyper::header::HOST;
use hyper::{Method, Request};
use profiles::_profile_id;
use std::sync::Arc;
use tracing::debug;
use tracing::error;
use tracing::warn;

use crate::server::context::GlobalContext;
use crate::server::server_request::{self, SR};
use crate::server::server_response;

pub const GET: &Method = &Method::GET;
pub const POST: &Method = &Method::POST;
pub const DELETE: &Method = &Method::DELETE;

const DEFAULT_DB: &str = "./sailboat.db";

#[rustfmt::skip]
pub async fn router(req: Request<Incoming>, g_ctx: Arc<GlobalContext<'_>>) -> ServerResponse {
    let path = req.uri().path();
    let host = req
        .headers()
        .get(HOST)
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
        db.query_row("SELECT value FROM globals WHERE key = 'domain'", (), |row| row.get(0))?
    };

    let req = server_request::get_request(req, &g_ctx, db, domain)?;

    // Serve static files separately
    // TODO: Refactor this so it happens before all the DB stuff
    if req.uri().path().starts_with("/static") {
        return serve_static::get(req).await;
    }

    // Remove the query parameter for routing purposes
    let without_query = match req.uri().path().split_once("?") {
        None => req.uri().path().to_owned(),
        Some(x) => x.0.to_owned(),
    };

    // Split into sub-routes
    let sub_routes: Vec<&str> = without_query.split("/").collect();
    let method = req.method().clone();

    match (&method, req, &sub_routes[1..]) {
        (GET,    SR::Setup(req),    ["profiles", "new"]) => profiles::new::get(req),
        (GET,    SR::Setup(req),    _) => profiles::new::redirect_to_create(req),

        (GET,    SR::Authed(req),   [""]) => index::get_authed(req).await,
        (GET,    SR::Plain(req),    [""]) => index::get(req),

        (GET,    req,               ["login"]) => login::get(req),
        (POST,   SR::Plain(req),    ["login"]) => login::post(req).await,
        (GET,    req,               ["logout"]) => logout::get(req),

        (GET,    SR::Authed(req),   ["feeds", _]) => _feed_handle::get(req).await,

        (POST,   SR::Authed(req),   ["follow"]) => follow::post(req).await,
        (GET,    SR::Authed(req),   ["following"]) => following::get(req).await,

        (POST,   SR::Authed(req),    ["profiles"]) => profiles::post(req.request).await,
        (POST,   SR::Setup(req),    ["profiles"]) => profiles::post(req).await,


        (GET,    SR::Authed(req),   ["profiles", _]) => _profile_id::get(req).await,
        // (GET, ["profiles", _, "outbox"]) => outbox::get(req),
        (GET,    SR::Plain(req),    ["switch", _]) => switch::get(req),

        (GET,    SR::Authed(req),   ["search", ..]) => search::get(req),
        (POST,   SR::Authed(req),   ["search", ..]) => search::post(req).await,

        (POST,   SR::Authed(req),   ["posts"]) => posts::post(req).await,
        (DELETE, SR::Authed(req),   ["posts", ..]) => posts::delete(req),

        (GET,    SR::Plain(req),    [".well-known", "webfinger"]) => webfinger::get(req).await,

        (GET,    SR::Authed(req),   ["debug"]) => debug::get(req),
        (GET,    SR::Authed(req),   ["healthcheck"]) => healthcheck::get(req),

        (_, req, _) => server_response::not_found(req),
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
            _ => log_error_and_send_generic_message(err),
        }
    } else {
        result
    }
}
