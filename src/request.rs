pub mod global_context;

use std::sync::Arc;
use crate::GlobalContext;
use crate::request::global_context::Context;
use core::fmt::Display;
use hyper::Request;
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
    Sql(rusqlite::Error)
}

impl std::error::Error for ServerError {}
impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ServerError::Hyper(ref err) => write!(f, "[HYPER ERROR] {}", err),
            ServerError::Sql(ref err) => write!(f, "[SQL ERROR] {}", err),
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

pub fn not_found(_req: Request<Incoming>, ctx: Context<'_>) -> ResponseResult {
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

pub fn ok(_: Request<Incoming>, _: Context<'_>) -> ResponseResult {
    Ok(send("OK".to_string()))
}
