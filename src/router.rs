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
use crate::server::error::{forbidden, ServerError};
use crate::server::server_response::ServerResult;
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
use crate::server::server_request::{AuthedRequest, AuthStatus, new_request, PlainRequest, SetupRequest};
use crate::server::server_response;

pub const GET: &Method = &Method::GET;
pub const POST: &Method = &Method::POST;
pub const DELETE: &Method = &Method::DELETE;

const DEFAULT_DB: &str = "./sailboat.db";

macro_rules! routes {
    (
        $req:ident,
        $method_to_match:ident,
        $sub_routes_to_match:ident, {
            $(($method:ident, $sub_routes:pat) => ($auth_func:ident, $handler:expr)),*
            $(,)?
        }
    ) => {
        match (&$method_to_match, $sub_routes_to_match) {
            $(
                ($method, $sub_routes) => {
                    let req = $auth_func($req)?;
                    $handler(req).await
                }
            )*
            _ => server_response::not_found($req),
        }
    };
}

#[rustfmt::skip]
pub async fn router(req: Request<Incoming>, g_ctx: Arc<GlobalContext<'_>>) -> ServerResult {
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

    let req = new_request(req, g_ctx, db, domain)?;

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
    let sub_routes = &sub_routes[1..];
    let method = req.method().clone();

    routes!(req, method, sub_routes, {
        (GET,       [""]) =>                            (any, index::get),

        (GET,       ["login"]) =>                       (any, login::get),
        (POST,      ["login"]) =>                       (any, login::post),
        (GET,       ["logout"]) =>                      (any, logout::get),

        (GET,       ["feeds", _]) =>                    (require_auth, _feed_handle::get),
        (POST,      ["follow"]) =>                      (require_auth, follow::post),
        (GET,       ["following"]) =>                   (require_auth, following::get),

        (POST,      ["profiles"]) =>                    (require_setup, profiles::post),
        (GET,       ["profiles", "new"]) =>             (require_setup, profiles::new::get),
        (GET,       ["profiles", _]) =>                 (any, _profile_id::get),

        (POST,      ["posts"]) =>                       (require_auth, posts::post),
        (DELETE,    ["posts", ..]) =>                   (require_auth, posts::delete),

        (GET,       ["switch", _]) =>                   (any, switch::get),
        (GET,       ["search", ..]) =>                  (require_auth, search::get),
        (POST,      ["search", ..]) =>                  (require_auth, search::post),

        (GET,       [".well-known", "webfinger"]) =>    (any, webfinger::get),

        (GET,       ["debug"]) =>                       (any, debug::get),
        (GET,       ["healthcheck"]) =>                 (any, healthcheck::get),
    })
}

fn any(req: PlainRequest) -> Result<PlainRequest, ServerError> {
    Ok(req)
}

fn require_auth(req: PlainRequest) -> Result<AuthedRequest, ServerError> {
    let req = require_setup(req)?;
    req.authenticate()
}

fn require_setup(req: PlainRequest) -> Result<SetupRequest, ServerError> {
    let req = req.to_setup();
    let req = match req {
        AuthStatus::Success(r) => r,
        AuthStatus::Failure(_) => return Err(forbidden())
    };
    Ok(req)
}

fn log_warn_and_send_specific_message(err: ServerError) -> ServerResult {
    warn!("{}", err);
    server_response::send_status_and_message(err)
}

fn log_error_and_send_generic_message(err: ServerError) -> ServerResult {
    error!("{}", err);
    server_response::send_status(err.status_code)
}

pub async fn serve(req: Request<Incoming>, g_ctx: Arc<GlobalContext<'_>>) -> ServerResult {
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
