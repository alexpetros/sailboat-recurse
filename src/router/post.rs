use crate::request::Request;
use crate::request::ResponseResult;
use serde::Serialize;
use crate::request;
use crate::request::global_context::Context;

#[derive(Debug, Serialize)]
struct Post {
    author_name: String,
    author_handle: String,
    content: String
}

pub async fn post(req: Request, _ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let body = req.body().to_vec();
    let text = String::from_utf8(body).unwrap();
    Ok(request::send(text))
}

