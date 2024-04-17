use crate::server::server_request::IncomingRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;

pub async fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let path = req.uri().path();
    if path.len() <= 8 {
        return server_response::not_found(req);
    }
    let file = &path[8..];

    let contents = req.global.statics.get(file);
    match contents {
        Some(body) => Ok(server_response::send(body.clone())),
        None => server_response::not_found(req),
    }
}
