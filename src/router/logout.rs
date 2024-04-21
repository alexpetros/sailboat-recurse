use crate::server::{server_request::SR, server_response::{redirect, ServerResponse}};

pub fn get<T>(req: SR<T>) -> ServerResponse {
    req.db().execute("DELETE FROM sessions", ())?;
    redirect("/")
}

// pub fn post(req: UnauthedRequest<'_>) -> ServerResponse {
// }
