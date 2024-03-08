use crate::request::global_context::GlobalContext;
use crate::request::ResponseResult;
use std::sync::Arc;
use hyper::body::Incoming;
use hyper::Request;

use crate::request;

pub fn get(req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.statics.get(file);

    match contents {
        Some(body) => Ok(request::send(body.clone())),
        None => request::not_found(req, ctx)
    }
}
