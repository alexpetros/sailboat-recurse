mod echo;

use hyper::body::Incoming;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use tracing::debug;

use crate::routes::echo::echo;
use crate::routes::echo::echo_upper;
use crate::routes::echo::echo_reversed;
use crate::request_utils::full;
use crate::request_utils::empty;

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    debug!("Serving request");
    match (req.method(), req.uri().path()) {
        (GET, "/") => hello(req),
        (POST, "/echo") => echo(req),
        (POST, "/echo/uppercase") => echo_upper(req),
        (POST, "/echo/reversed") => echo_reversed(req).await,

        // Return 404 otherwise
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn hello(_: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(full("Hello, World!\n")))
}

