use minijinja::context;

use crate::server::{server_request::UnauthedRequest, server_response::{self, ServerResponse}};

pub fn get(req: UnauthedRequest<'_>) -> ServerResponse {
    let body = req.render("profiles/new.html", context! {})?;
    Ok(server_response::send(body))
}
