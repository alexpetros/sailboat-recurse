use minijinja::context;
use hyper::{Response, StatusCode};
use hyper::body::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use std::sync::Arc;
use crate::server::context::{Context, GlobalContext};
use crate::server::request::Request;
use crate::server::error::ServerError;

pub type ResponseResult = std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, ServerError>;


fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}

pub fn send<T: Into<Bytes>>(body: T) -> Response<BoxBody<hyper::body::Bytes, hyper::Error>> {
    Response::new(full(body))
}

pub fn send_status(status: StatusCode) -> ResponseResult {
    let mut res = Response::new(empty());
    *res.status_mut() = status;
    Ok(res)
}

pub fn not_found(_req: Request, ctx: Context<'_>) -> ResponseResult {
    let page = ctx.render("404.html", context! {});
    let mut res = send(page);
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}

pub fn _server_error(g_ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let page = g_ctx.render("500.html", context! {});
    let mut res = send(page);
    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    Ok(res)
}

pub fn ok() -> ResponseResult {
    Ok(send("OK".to_string()))
}
