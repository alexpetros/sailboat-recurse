use minijinja::context;

use crate::server::{server_request::SetupRequest, server_response::{self, redirect, ServerResponse}};

pub fn get(req: SetupRequest<'_>) -> ServerResponse {
    let body = req.render("profiles/new.html", context! {})?;
    Ok(server_response::send(body))
}

pub fn redirect_to_create(_req: SetupRequest<'_>) -> ServerResponse {
    redirect("/profiles/new")
}
