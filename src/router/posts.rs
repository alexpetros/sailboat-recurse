use crate::server::error;
use crate::server::error::body_not_utf8;
use minijinja::context;
use crate::queries::get_posts_in_feed;
use tracing::log::debug;
use serde::Deserialize;
use serde::Serialize;
use crate::server::request::{IncomingRequest};
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

pub async fn post(req: IncomingRequest<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: PostForm = serde_html_form::from_str(&text)?;
    let feed_id: i64 = form.feed_id.parse().map_err(|_| { body_not_utf8() })?;
    req.db.execute(
        "INSERT INTO posts (feed_id, content) VALUES (?1, ?2)",
        (&feed_id, &form.content)
    )?;

    let posts = get_posts_in_feed(&req.db, feed_id)?;
    let body = req.render_block("feed.html", "feed", context! { posts });
    Ok(send(body))
}

pub fn delete(req: IncomingRequest<'_>) -> ResponseResult {
    let post_param = req.uri().path().split("/")
        .nth(2)
        .ok_or(error::bad_request("Missing ID"))?;

    debug!("Deleting post {}", post_param);
    req.db.execute("DELETE FROM posts WHERE post_id = ?1", [ post_param ])?;
    Ok(send("".to_owned()))
}
