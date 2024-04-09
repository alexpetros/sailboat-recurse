use crate::server::request::IncomingRequest;
use crate::server::response;
use crate::server::response::ResponseResult;

pub fn get (req: IncomingRequest<'_>) -> ResponseResult {
    let body = req.global.startup_time.to_string();
    Ok(response::send(body))
}
