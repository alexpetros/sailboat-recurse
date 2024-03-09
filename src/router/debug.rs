use crate::request::ResponseResult;
use crate::request;
use crate::request::global_context::Context;
use hyper::Request;
use hyper::body::Incoming;

pub fn get(_req: Request<Incoming>, ctx: Context<'_>) -> ResponseResult {
    let body = ctx.global.startup_time.to_string();
    Ok(request::send(body))
}

