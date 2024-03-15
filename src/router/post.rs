use minijinja::context;
use crate::queries::feed::get_posts_in_feed;
use tracing::log::debug;
use crate::server::error::ServerError;
use serde::Deserialize;
use serde::Serialize;
use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::response::{ResponseResult, send};

#[derive(Debug, Serialize)]
struct Post {
    author_name: String,
    author_handle: String,
    content: String
}

#[derive(Debug, Deserialize)]
struct PostForm {
    feed_id: String,
    content: String
}

pub async fn post(req: Request, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: PostForm = serde_html_form::from_str(&text).unwrap();
    let feed_id: i64 = form.feed_id.parse().map_err(|_| { ServerError::BodyNotUtf8() })?;
    ctx.db.execute(
        "INSERT INTO posts (feed_id, content) VALUES (?1, ?2)",
        (&feed_id, &form.content)
    )?;

    let posts = get_posts_in_feed(&ctx)?;
    let body = ctx.render_block("index.html", "feed", context! { posts });
    Ok(send(body))
}

pub fn delete(req: Request, ctx: Context<'_>) -> ResponseResult {
    let post_param = req.uri().path().split("/")
        .nth(2)
        .ok_or(ServerError::BadRequest("Missing ID".to_owned()))?;

    debug!("Deleting post {}", post_param);
    ctx.db.execute("DELETE FROM posts WHERE post_id = ?1", [ post_param ])?;
    Ok(send("".to_owned()))
}
