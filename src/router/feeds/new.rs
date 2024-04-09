use minijinja::context;

use crate::server::request::IncomingRequest;
use crate::server::response::{self, ResponseResult};

pub fn get(req: IncomingRequest<'_>) -> ResponseResult {
    let body = req.render("feeds/new.html", context! {});
    Ok(response::send(body))
}
