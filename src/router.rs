mod index;
mod post;
mod debug;
mod serve_static;
mod healthcheck;

use crate::server::context::Context;
use crate::server::response::ResponseResult;
use crate::sqlite::get_conn;
use std::sync::Arc;
use tracing::debug;
use tracing::error;

use crate::server::request::Request;
use crate::server::context::GlobalContext;
use crate::server::response;

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

    let sub_route = path.split("/").nth(1);

    let result = match sub_route {
        Some("") => index::router(req, ctx),
        Some("post") => post::router(req, ctx).await,
        Some("debug") => debug::router(req, ctx),
        Some("healthcheck") => healthcheck::router(req, ctx),

        // Return 404 if the request is not known
        _ => response::not_found(req, ctx)
    };

    if let Err(error) = result {
        error!("{}", error);
        response::server_error(g_ctx)
    } else {
        result
    }
}

