use crate::queries::get_posts_in_feed;
use crate::server::context::Context;
use minijinja::context;
use crate::server::request::IncomingRequest;
use crate::server::response;
use crate::server::response::ResponseResult;

pub async fn get(_req: IncomingRequest, ctx: Context<'_>) -> ResponseResult {
    let posts = get_posts_in_feed(&ctx.db, 1)?;


    let context = context! {
        posts,
        feed_id => "1",
        name => "Alex",
        bio => "Rigging my sailboat"
    };
    let body = ctx.render("index.html", context);
    Ok(response::send(body))
}
