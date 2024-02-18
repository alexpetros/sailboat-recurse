use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
// use tracing::{error, info};

pub async fn router (req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    return hello(req);
}

fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let hello_bytes = Bytes::from("Hello, World!\n");
    Ok(Response::new(Full::new(hello_bytes)))
}

