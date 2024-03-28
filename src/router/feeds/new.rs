use minijinja::context;

use crate::server::context::Context;
use crate::server::request::IncomingRequest;
use crate::server::response::{self, ResponseResult};

pub fn get(_req: IncomingRequest, ctx: Context<'_>) -> ResponseResult {
    let body = ctx.render("feeds/new.html", context! {});
    Ok(response::send(body))
}
