use crate::Context;
use std::sync::Arc;
use crate::request_utils::full;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};
use minijinja::Environment;

pub fn get(_req: Request<Incoming>, ctx: Arc<Context<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let body = ctx.statics.get("common.css").unwrap().clone();

    Ok(Response::new(full(body)))
}
