pub mod global_context;

use std::ops::Deref;
use std::sync::Arc;
use crate::GlobalContext;
use crate::request::global_context::Context;
use core::fmt::Display;
use hyper::body::Incoming;
use minijinja::context;
use hyper::StatusCode;
use hyper::Response;
use http_body_util::Empty;
use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;

use http_body_util::BodyExt;

#[derive(Debug)]
pub enum ServerError {
    Hyper(hyper::Error),
    Sql(rusqlite::Error),
    BodyTooLarge()
}

impl std::error::Error for ServerError {}
impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ServerError::Hyper(ref err) => write!(f, "[HYPER ERROR] {}", err),
            ServerError::Sql(ref err) => write!(f, "[SQL ERROR] {}", err),
            ServerError::BodyTooLarge() => write!(f, "Body Too Large Error"),
        }
    }
}

impl From<rusqlite::Error> for ServerError {
    fn from(err: rusqlite::Error) -> Self {
        ServerError::Sql(err)
    }
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> Self {
        ServerError::Hyper(err)
    }
}

pub struct Request(pub hyper::Request<Incoming>);
impl From<hyper::Request<Incoming>> for Request {
    fn from(req: hyper::Request<Incoming>) -> Self {
        Request(req)
    }
}

impl Request {
    pub async fn get_body(self) -> Result<FullRequest, ServerError> {
        let (parts, body) = self.0.into_parts();
        let body_bytes = http_body_util::Limited::new(body, 1024 * 64);

        let bytes = body_bytes.collect().await
            .map(|r| { r.to_bytes() })
            .map_err(|_| { ServerError::BodyTooLarge() })?;

        let req = hyper::Request::from_parts(parts, bytes);
        Ok(FullRequest(req))
    }
}

impl Deref for Request {
    type Target = hyper::Request<Incoming>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FullRequest(pub hyper::Request<Bytes>);
impl From<hyper::Request<Bytes>> for FullRequest {
    fn from(req: hyper::Request<Bytes>) -> Self {
        FullRequest(req)
    }
}

impl Deref for FullRequest {
    type Target = hyper::Request<Bytes>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



impl Request {
}

pub type ResponseResult = std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, ServerError>;


fn _empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}

pub fn send<T: Into<Bytes>>(body: T) -> Response<BoxBody<hyper::body::Bytes, hyper::Error>> {
    Response::new(full(body))
}

pub async fn send_async<T: Into<Bytes>>(body: T) -> ResponseResult {
    Ok(send(body))
}


pub async fn not_found(_req: Request, ctx: Context<'_>) -> ResponseResult {
    let page = ctx.render("404.html", context! {});
    let mut res = send(page);
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}

pub fn server_error(g_ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let page = g_ctx.render("500.html", context! {});
    let mut res = send(page);
    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    Ok(res)
}

pub fn ok() -> ResponseResult {
    Ok(send("OK".to_string()))
}
