use crate::server::{server_request::AnyRequest, server_response::{redirect, ServerResult}};
use crate::server::server_request::AuthState;

pub async fn get<'a, Au: AuthState>(req: AnyRequest<'a, Au>) -> ServerResult {
    req.db.execute("DELETE FROM sessions", ())?;
    redirect("/")
}

// pub fn post(req: UnauthedRequest<'_>) -> ServerResponse {
// }
