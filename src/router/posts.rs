use crate::activitypub::objects::note::get_post;
use crate::query_row_custom;
use crate::server::error::body_not_utf8;
use crate::server::server_request::{AnyRequest, AuthState, AuthedRequest};
use crate::server::server_response::{send, ServerResult};
use crate::templates::_partials::post::Post;
use minijinja::context;
use serde::Deserialize;
use serde_json::json;
use tracing::log::debug;

#[derive(Debug, Deserialize)]
struct PostForm {
    profile_id: String,
    content: String,
}

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
    let post = get_post(&req.db, post_id, &req.domain)?;
    let body = json!(post).to_string();
    Ok(send(body))
}

pub async fn post(req: AuthedRequest<'_>) -> ServerResult {
    let req = req.into_text().await?;
    let form: PostForm = req.get_form_data()?;
    let profile_id: i64 = form.profile_id.parse().map_err(|_| body_not_utf8())?;

    req.db.execute(
        "INSERT INTO posts (profile_id, content) VALUES (?1, ?2)",
        (&profile_id, &form.content),
    )?;
    let post_id = req.db.last_insert_rowid();

    let post: Post = req.db.query_row(
        "
        SELECT post_id, content, created_at, display_name, preferred_username
        FROM posts
        LEFT JOIN profiles USING (profile_id)
        WHERE post_id = ?1
        ",
        (post_id,),
        |row| {
            let post = Post {
                post_id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
                actor_name: row.get(3)?,
                actor_handle: row.get(4)?,
            };
            Ok(post)
        },
    )?;

    let body = req.render("_partials/post.html", context! { post })?;

    Ok(send(body))
}

pub async fn delete(req: AuthedRequest<'_>) -> ServerResult {
    let post_param = req.get_url_param(2, "Missing post ID")?;
    debug!("Deleting post {}", post_param);
    req.db.execute("DELETE FROM posts WHERE post_id = ?1", [post_param])?;
    Ok(send("".to_owned()))
}
