use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::header::{HeaderValue, LOCATION};
use hyper::{Response, StatusCode};
use minijinja::context;

use crate::server::error::ServerError;
use crate::server::server_request::IncomingRequest;

pub type ServerResponse = Result<Response<BoxBody<Bytes, hyper::Error>>, ServerError>;

pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn send<T: Into<Bytes>>(body: T) -> Response<BoxBody<hyper::body::Bytes, hyper::Error>> {
    Response::new(full(body))
}

pub fn redirect(path: &str) -> ServerResponse {
    let mut res = Response::new(empty());
    let location_val = HeaderValue::from_str(path).map_err(|_| ServerError {
        prefix: "[HEADER ERROR]",
        message: "Invalid Redirect Provided".to_owned(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    *res.status_mut() = StatusCode::SEE_OTHER;
    res.headers_mut().insert(LOCATION, location_val);
    Ok(res)
}

pub fn send_status(status: StatusCode) -> ServerResponse {
    let mut res = Response::new(empty());
    *res.status_mut() = status;
    Ok(res)
}

pub fn send_status_and_message(error: ServerError) -> ServerResponse {
    let mut res = Response::new(full(error.message));
    *res.status_mut() = error.status_code;
    Ok(res)
}

pub fn not_found(req: IncomingRequest<'_>) -> ServerResponse {
    let page = req.render("404.html", context! {})?;
    let mut res = send(page);
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}

pub fn ok() -> ServerResponse {
    Ok(send("OK".to_string()))
}
