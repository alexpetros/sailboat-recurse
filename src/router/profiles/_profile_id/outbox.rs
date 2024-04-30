use serde::Deserialize;
use serde_json::json;

use crate::{activitypub::objects::outbox::{get_outbox, get_outbox_page}, server::{server_request::{AnyRequest, AuthState}, server_response::{send, ServerResult}}};


#[derive(Debug, Deserialize)]
struct Query {
    page: usize
}

pub async fn get<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let profile_id = req.get_int_url_param(2, "Missing profile ID")?;

    let query = req.uri()
        .query()
        .map(serde_html_form::from_str::<Query>)
        .and_then(|r| r.ok());

    if let Some(q) = query {
        let page_num = q.page;
        let outbox_page = get_outbox_page(&req.db, profile_id, &req.domain, page_num)?;
        let body = json!(outbox_page).to_string();
        Ok(send(body))
    } else {
        let outbox = get_outbox(&req.db, profile_id, &req.domain)?;
        let body = json!(outbox).to_string();
        Ok(send(body))
    }

}
