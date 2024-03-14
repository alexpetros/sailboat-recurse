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

pub async fn post(req: Request, _ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    Ok(send(text))
}

