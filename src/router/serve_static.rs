use crate::request::global_context::Context;
use crate::request::ResponseResult;
use hyper::body::Incoming;
use hyper::Request;

use crate::request;

pub fn get(req: Request<Incoming>, ctx: Context<'_>) -> ResponseResult {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.global.statics.get(file);

    match contents {
        Some(body) => Ok(request::send(body.clone())),
        None => request::not_found(req, ctx)
    }
}
