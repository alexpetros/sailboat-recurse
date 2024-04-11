use crate::server::server_request::IncomingRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;

pub fn get (_req: IncomingRequest<'_>) -> ServerResponse {
    server_response::ok()
}


