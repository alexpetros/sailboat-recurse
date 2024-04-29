use serde_json::json;

use crate::{activitypub::objects::outbox::get_outbox, server::{server_request::{AnyRequest, AuthState}, server_response::{send, ServerResult}}};


pub async fn get<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let profile_id = 1;
    let outbox = get_outbox(&req.db, profile_id, &req.domain)?;
    let body = json!(outbox).to_string();
    Ok(send(body))
}
