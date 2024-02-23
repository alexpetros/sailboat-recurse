use hyper::body::Body;
use hyper::body::Frame;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

use crate::request_utils::full;

pub fn echo(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let res = Response::new(req.into_body().boxed());
    Ok(res)
}

pub fn echo_upper(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let frame_stream = req.into_body().map_frame(|frame| {
        let frame = if let Ok(data) = frame.into_data() {
            data.iter().map(|byte| byte.to_ascii_uppercase()).collect::<Bytes>()
        } else {
            Bytes::new()
        };

        Frame::data(frame)
    });

    Ok(Response::new(frame_stream.boxed()))
}

pub async fn echo_reversed(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Bodies should only be so large
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 1024 * 64 {
        let mut res = Response::new(full("Body too large"));
        *res.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(res)
    }

    // Await the whole body
    let whole_body = req.collect().await?.to_bytes();

    let reversed_body = whole_body.iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>();

    Ok(Response::new(full(reversed_body)))
}
