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
use crate::server::server_response::{redirect, ServerResult};
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
use crate::server::server_request::{new_request, AuthStatus, AuthedRequest, PlainRequest, SetupRequest, SetupStatus};
use crate::server::server_response;

pub const GET: &Method = &Method::GET;
pub const POST: &Method = &Method::POST;
pub const DELETE: &Method = &Method::DELETE;

const DEFAULT_DB: &str = "./sailboat.db";

pub enum MiddlewareResult<T> {
    Continue(T),
    Finish(ServerResult)
}

macro_rules! routes {
    (
        $req:ident,
        $method_to_match:ident,
        $sub_routes_to_match:ident, {
            $(($method:pat, $sub_routes:pat) => ($auth_func:ident, $handler:expr)),*
            $(,)?
        }
    ) => {
        match (&$method_to_match, $sub_routes_to_match) {
            $(
                ($method, $sub_routes) => {
                    let req = match $auth_func($req) {
                        MiddlewareResult::Continue(r) => r,
                        MiddlewareResult::Finish(e) => return e
                    };
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

        (GET,       ["feeds", _]) =>                    (require_full_setup, _feed_handle::get),
        (POST,      ["follow"]) =>                      (require_full_setup, follow::post),
        (GET,       ["following"]) =>                   (require_full_setup, following::get),

        (POST,      ["profiles"]) =>                    (require_authentication, profiles::post),
        (GET,       ["profiles", "new"]) =>             (require_authentication, profiles::new::get),
        (GET,       ["profiles", _]) =>                 (any, _profile_id::get),

        (POST,      ["posts"]) =>                       (require_full_setup, posts::post),
        (DELETE,    ["posts", ..]) =>                   (require_full_setup, posts::delete),

        (GET,       ["switch", _]) =>                   (any, switch::get),
        (GET,       ["search", ..]) =>                  (require_full_setup, search::get),
        (POST,      ["search", ..]) =>                  (require_full_setup, search::post),

        (GET,       [".well-known", "webfinger"]) =>    (any, webfinger::get),

        (GET,       ["debug"]) =>                       (any, debug::get),
        (GET,       ["healthcheck"]) =>                 (any, healthcheck::get),
    })
}

fn any(req: PlainRequest) -> MiddlewareResult<PlainRequest> {
    MiddlewareResult::Continue(req)
}

fn require_full_setup(req: PlainRequest) -> MiddlewareResult<AuthedRequest> {
    let req = require_authentication(req);
    let req = match req {
        MiddlewareResult::Continue(r) => r,
        MiddlewareResult::Finish(e) => return MiddlewareResult::Finish(e)
    };

    let req = match req.has_passed_setup() {
        Ok(r) => r,
        Err(e) => return MiddlewareResult::Finish(Err(e))
    };

    match req {
        SetupStatus::Complete(r) => MiddlewareResult::Continue(r),
        SetupStatus::Incomplete(_) => MiddlewareResult::Finish(redirect("/profiles/new"))
    }
}

fn require_authentication(req: PlainRequest) -> MiddlewareResult<SetupRequest> {
    let req = req.authenticate();
    match req {
        AuthStatus::Success(r) => MiddlewareResult::Continue(r),
        AuthStatus::Failure(_) => MiddlewareResult::Finish(Err(forbidden()))
    }
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
