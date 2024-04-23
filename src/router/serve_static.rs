use crate::server::server_request::AnyRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;

pub async fn get(req: AnyRequest<'_>) -> ServerResponse {
    let path = req.uri().path();
    if path.len() <= 8 {
        return server_response::not_found(req);
    }
    let file = &path[8..];

    let statics = &req.global.statics;
    let contents = statics.get(file);
    match contents {
        Some(body) => Ok(server_response::send(body.clone())),
        None => server_response::not_found(req),
    }
}
