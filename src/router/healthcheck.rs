use crate::server::server_request::AuthedRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;

pub fn get(_req: AuthedRequest<'_>) -> ServerResponse {
    server_response::ok()
}
