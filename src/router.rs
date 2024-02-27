mod index;
mod echo;

use hyper::body::Incoming;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use tracing::debug;

use crate::router::echo::echo;
use crate::router::echo::echo_upper;
use crate::router::echo::echo_reversed;
use crate::request_utils::full;
use crate::request_utils::empty;

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let method = req.method();
    let path = req.uri().path();

    debug!("Received {} request at {}", method, path);
    match (method, path) {
        (GET, "/healthcheck") => healthcheck(req),
        (GET, "/") => index::get(req),
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

fn healthcheck(_: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(full("OK")))
}

