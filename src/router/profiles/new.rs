use minijinja::context;

use crate::server::{server_request::SetupRequest, server_response::{self, ServerResult}};

pub async fn get(req: SetupRequest<'_>) -> ServerResult {
    let body = req.render("profiles/new.html", context! {})?;
    Ok(server_response::send(body))
}
