use crate::request_utils;
use crate::Context;
use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub fn get(req: Request<Incoming>, ctx: Arc<Context<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.statics.get(file);

    match contents {
        Some(body) => Ok(request_utils::send(body.clone())),
        None => Ok(request_utils::not_found())
    }
}
