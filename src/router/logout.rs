use crate::server::{server_request::AnyRequest, server_response::{redirect, ServerResponse}};

pub fn get(req: AnyRequest) -> ServerResponse {
    req.db.execute("DELETE FROM sessions", ())?;
    redirect("/")
}

// pub fn post(req: UnauthedRequest<'_>) -> ServerResponse {
// }
