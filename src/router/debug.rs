use crate::request::ResponseResult;
use crate::request;
use crate::request::global_context::GlobalContext;
use hyper::Request;
use hyper::body::Incoming;
use std::sync::Arc;

pub fn get(_req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let body = ctx.startup_time.to_string();
    Ok(request::send(body))
}

