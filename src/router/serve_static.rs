use crate::server::request::IncomingRequest;
use crate::server::response;
use crate::server::response::ResponseResult;

pub async fn get(req: IncomingRequest<'_>) -> ResponseResult {
    let path = req.uri().path();
    if path.len() <= 8 {
        return response::not_found(req);
    }
    let file = &path[8..];

    let contents = req.global.statics.get(file);
    match contents {
        Some(body) => Ok(response::send(body.clone())),
        None => response::not_found(req)
    }
}
