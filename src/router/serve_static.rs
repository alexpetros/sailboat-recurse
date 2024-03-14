use crate::server::context::Context;
use crate::server::response::ResponseResult;

use crate::server::request::Request;
use crate::server::response;

pub async fn get(req: Request, ctx: Context<'_>) -> ResponseResult {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.global.statics.get(file);

    match contents {
        Some(body) => response::send_async(body.clone()).await,
        None => response::not_found(req, ctx).await
    }
}
