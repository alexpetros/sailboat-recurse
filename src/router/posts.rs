use crate::server::error;
use crate::server::error::body_not_utf8;
use minijinja::context;
use crate::queries::get_posts_in_profile;
use tracing::log::debug;
use serde::Deserialize;
use serde::Serialize;
use crate::server::server_request::{IncomingRequest};
use crate::server::server_response::{ServerResponse, send};

#[derive(Debug, Serialize)]
struct Post {
    author_name: String,
    author_handle: String,
    content: String
}

#[derive(Debug, Deserialize)]
struct PostForm {
    profile_id: String,
    content: String
}

pub async fn post(req: IncomingRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
    let form: PostForm = req.get_form_data()?;
    let profile_id: i64 = form.profile_id.parse().map_err(|_| { body_not_utf8() })?;
    req.db.execute(
        "INSERT INTO posts (profile_id, content) VALUES (?1, ?2)",
        (&profile_id, &form.content)
    )?;

    let posts = get_posts_in_profile(&req.db, profile_id)?;
    let body = req.render_block("profile.html", "profile", context! { posts });
    Ok(send(body))
}

pub fn delete(req: IncomingRequest<'_>) -> ServerResponse {
    let post_param = req.uri().path().split("/")
        .nth(2)
        .ok_or(error::bad_request("Missing ID"))?;

    debug!("Deleting post {}", post_param);
    req.db.execute("DELETE FROM posts WHERE post_id = ?1", [ post_param ])?;
    Ok(send("".to_owned()))
}
