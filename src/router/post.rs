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
    content: String
}

const AUTHOR_NAME: &str = "Alex Petros";
const AUTHOR_HANDLE: &str = "awp@example.com";

pub async fn post(req: Request, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: PostForm = serde_html_form::from_str(&text).unwrap();
    ctx.db.execute(
        "INSERT INTO posts (author_name, author_handle, content) VALUES (?1, ?2, ?3)",
        (&AUTHOR_NAME, &AUTHOR_HANDLE, &form.content)
    )?;

    Ok(send("".to_owned()))
}

// pub async fn delete(req: Request, ctx: Context<'_>) -> ResponseResult {
// }