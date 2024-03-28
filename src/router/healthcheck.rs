use crate::server::context::Context;
use crate::server::request::IncomingRequest;
use crate::server::{response};
use crate::server::response::ResponseResult;

pub fn get (_req: IncomingRequest, _ctx: Context<'_>) -> ResponseResult {
    response::ok()
}


