use crate::queries::feed::get_posts_in_feed;
use crate::server::context::Context;
use minijinja::context;
use crate::server::request::{GET, Request};
use crate::server::response;
use crate::server::response::{not_found, ResponseResult};

pub fn router(req: Request, ctx: Context<'_>) -> ResponseResult {
    if req.method() != GET { return not_found(req, ctx) }

    let posts = get_posts_in_feed(&ctx)?;

    let context = context! {
        posts,
        name => "Alex",
        bio => "Rigging my sailboat"
    };
    let body = ctx.render("index.html", context);
    Ok(response::send(body))
}
