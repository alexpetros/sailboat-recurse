use crate::request::ResponseResult;
use crate::request;
use crate::request::Request;
use crate::request::global_context::Context;

pub fn get(_req: Request, ctx: Context<'_>) -> ResponseResult {
    let body = ctx.global.startup_time.to_string();
    Ok(request::send(body))
}

