use serde::Deserialize;

use crate::server::context::Context;
use crate::server::response::send;
use crate::server::{request::IncomingRequest, response::ResponseResult};

#[derive(Deserialize)]
struct Query {
    q: String
}

pub async fn post(req: IncomingRequest, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;

    let query = serde_html_form::from_str::<Query>(&text)?.q;

    // let posts = get_posts_in_feed(&ctx.db, feed_id)?;
    // let body = ctx.render_block("feed.html", "feed", context! { posts });
    let body = "".to_owned();
    Ok(send(body))
}
