use crate::request;
use crate::request::global_context::GlobalContext;
use hyper::{Request, Response};
use hyper::body::Bytes;
use hyper::body::Incoming;
use http_body_util::combinators::BoxBody;
use std::sync::Arc;

pub fn get(_req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let body = ctx.startup_time.to_string();
    Ok(request::send(body))
}

