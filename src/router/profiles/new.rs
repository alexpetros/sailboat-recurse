use minijinja::context;

use crate::server::{server_request::SR, server_response::{self, ServerResponse}};

pub fn get<T> (req: SR<T>) -> ServerResponse {
    let body = req.render("profiles/new.html", context! {})?;
    Ok(server_response::send(body))
}
