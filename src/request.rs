pub mod global_context;

use hyper::Request;
use hyper::body::Incoming;
use minijinja::context;
use std::sync::Arc;
use crate::GlobalContext;
use hyper::StatusCode;
use hyper::Response;
use http_body_util::Empty;
use http_body_util::Full;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Bytes;

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn _empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}

pub fn send<T: Into<Bytes>>(body: T) -> Response<BoxBody<hyper::body::Bytes, hyper::Error>> {
    Response::new(full(body))
}

pub fn not_found(_req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let page = ctx.render("404.html", context! {});
    let mut res = send(page);

    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}

