use crate::server::request::IncomingRequest;
use crate::server::{response};
use crate::server::response::ResponseResult;

pub fn get (_req: IncomingRequest<'_>) -> ResponseResult {
    response::ok()
}


