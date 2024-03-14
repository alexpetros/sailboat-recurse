use crate::request::global_context::Context;
use crate::request::ResponseResult;

use crate::request;
use crate::request::Request;

pub async fn get(req: Request, ctx: Context<'_>) -> ResponseResult {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.global.statics.get(file);

    match contents {
        Some(body) => request::send_async(body.clone()).await,
        None => request::not_found(req, ctx).await
    }
}
