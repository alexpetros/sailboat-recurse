use crate::Context;
use hyper::StatusCode;
use crate::request_utils::empty;
use std::sync::Arc;
use crate::request_utils::full;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub fn get(req: Request<Incoming>, ctx: Arc<Context<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let path = req.uri().path();
    let file = &path[8..];
    let contents = ctx.statics.get(file);

    match contents {
        Some(body) => Ok(Response::new(full(body.clone()))),
        None => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }


}
