use tracing::info;
use http_body_util::Empty;
use http_body_util::Full;
use hyper::body::Frame;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Bytes;
use hyper::{Request, Response};
use tracing::debug;

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    debug!("Serving request");
    match (req.method(), req.uri().path()) {
        (GET, "/") => hello(req),
        (POST, "/echo") => echo(req),

        // Return 404 otherwise
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(full("Hello, World!\n")))
}

fn echo(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let res = Response::new(req.into_body().boxed());
    debug!("{:?}", &res);
    Ok(res)
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}

