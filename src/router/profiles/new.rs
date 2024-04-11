use minijinja::context;

use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{self, ServerResponse};

pub fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let body = req.render("profiles/new.html", context! {});
    Ok(server_response::send(body))
}
