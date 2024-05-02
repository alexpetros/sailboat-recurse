use crate::activitypub::objects::note::{get_post, Note};
use crate::query_row_custom;
use crate::server::server_request::{AnyRequest, AuthState};
use crate::server::server_response::{send, ServerResult};

use minijinja::context;
use serde_json::json;

pub async fn get<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    match req.is_ap_req() {
        true => get_json(req),
        false => get_html(req)
    }
}

fn get_html<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let post_id = req.get_url_param(2, "Missing post ID")?;
    let post = query_row_custom!(
        req.db,
        Post {
            post_id: i64,
            actor_name: String,
            actor_handle: String,
            content: String,
            created_at: String,
            display_name: String,
            preferred_username: String
        },
        "
        SELECT
            post_id,
            display_name as actor_name,
            preferred_username as actor_handle,
            content,
            created_at,
            display_name,
            preferred_username
        FROM posts
        LEFT JOIN profiles USING (profile_id)
        WHERE post_id = ?1
        ",
        [post_id])?;

    let body = req.render("posts/_post_id.html", context! { post })?;
    Ok(send(body))
}

fn get_json<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let post_id = req.get_url_param(2, "Missing post ID")?;
    let note: Note = get_post(&req.db, post_id, &req.domain)?.into();
    let body = json!(note).to_string();
    Ok(send(body))
}

