use crate::server::server_request::AuthedRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;

pub fn get(req: AuthedRequest<'_>) -> ServerResponse {
    let body = req.global.startup_time.to_string();
    Ok(server_response::send(body))
}
