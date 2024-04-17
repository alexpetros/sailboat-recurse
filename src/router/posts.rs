use crate::server::error;
use crate::server::error::body_not_utf8;
use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{send, ServerResponse};
use crate::templates::_partials::post::Post;
use minijinja::context;
use serde::Deserialize;
use tracing::log::debug;

#[derive(Debug, Deserialize)]
struct PostForm {
    profile_id: String,
    content: String,
}

pub async fn post(req: IncomingRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
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

pub fn delete(req: IncomingRequest<'_>) -> ServerResponse {
    let post_param = req
        .uri()
        .path()
        .split('/')
        .nth(2)
        .ok_or(error::bad_request("Missing ID"))?;

    debug!("Deleting post {}", post_param);
    req.db
        .execute("DELETE FROM posts WHERE post_id = ?1", [post_param])?;
    Ok(send("".to_owned()))
}
