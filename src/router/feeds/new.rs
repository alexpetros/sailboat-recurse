use minijinja::context;

use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::response::{self, ResponseResult};

pub fn get(_req: Request, ctx: Context<'_>) -> ResponseResult {
    let body = ctx.render("feeds/new.html", context! {});
    Ok(response::send(body))
}
