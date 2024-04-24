use crate::server::server_request::PlainRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResult;

pub async fn get(req: PlainRequest<'_>) -> ServerResult {
    let body = req.global.startup_time.to_string();
    Ok(server_response::send(body))
}
