use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::{response};
use crate::server::response::ResponseResult;

pub fn get (_req: Request, _ctx: Context<'_>) -> ResponseResult {
    response::ok()
}


