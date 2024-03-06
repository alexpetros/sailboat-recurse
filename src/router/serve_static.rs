use crate::request::global_context::GlobalContext;
use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

use crate::request;

pub fn get(req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.statics.get(file);

    match contents {
        Some(body) => Ok(request::send(body.clone())),
        None => request::not_found(req, ctx)
    }
}
