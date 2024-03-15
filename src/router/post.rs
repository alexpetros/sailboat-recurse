use tracing::log::debug;
use crate::server::error::ServerError;
use serde::Deserialize;
use serde::Serialize;
use request::{DELETE, POST};
use crate::server::context::Context;
use crate::server::{request, response};
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
    content: String
}
pub async fn router (req: Request, ctx: Context<'_>) -> ResponseResult {
    match (req.method(), req.uri().path()) {
        (POST, "/post") => post(req, ctx).await,
        (DELETE, _) => delete(req, ctx),
        _ => response::not_found(req, ctx)
    }
}

const AUTHOR_NAME: &str = "Alex Petros";
const AUTHOR_HANDLE: &str = "awp@example.com";

async fn
post(req: Request, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: PostForm = serde_html_form::from_str(&text).unwrap();
    ctx.db.execute(
        "INSERT INTO posts (author_name, author_handle, content) VALUES (?1, ?2, ?3)",
        (&AUTHOR_NAME, &AUTHOR_HANDLE, &form.content)
    )?;

    Ok(send("".to_owned()))
}

pub fn delete(req: Request, ctx: Context<'_>) -> ResponseResult {
    let post_param = req.uri().path().split("/")
        .nth(2)
        .ok_or(ServerError::BadRequest("Missing ID".to_owned()))?;

    debug!("Deleting post {}", post_param);
    ctx.db.execute("DELETE FROM posts WHERE post_id = ?1", [ post_param ])?;
    Ok(send("".to_owned()))
}
