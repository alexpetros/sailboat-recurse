use crate::server::server_request::PlainRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResult;

pub async fn get(_req: PlainRequest<'_>) -> ServerResult {
    server_response::ok()
}
