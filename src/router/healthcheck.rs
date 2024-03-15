use crate::server::context::Context;
use crate::server::request::{GET, Request};
use crate::server::{response};
use crate::server::response::{not_found, ResponseResult};

pub fn router (req: Request, ctx: Context<'_>) -> ResponseResult {
    if req.method() != GET { return not_found(req, ctx) }

    response::ok()
}


