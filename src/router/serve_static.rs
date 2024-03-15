use crate::server::context::Context;
use crate::server::response::ResponseResult;

use crate::server::request::Request;
use crate::server::response;

pub async fn get(req: Request, ctx: Context<'_>) -> ResponseResult {
    let path = req.uri().path();
    if path.len() <= 8 {
        return response::not_found(req, ctx);
    }
    let file = &path[8..];

    let contents = ctx.global.statics.get(file);
    match contents {
        Some(body) => Ok(response::send(body.clone())),
        None => response::not_found(req, ctx)
    }
}
