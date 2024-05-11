use crate::activitypub::objects::note::get_post;
use crate::activitypub::requests::send_as;
use crate::query_map;
use crate::router::debug;
use crate::server::error::body_not_utf8;
use crate::server::server_request::AuthedRequest;
use crate::server::server_response::{send, ServerResult};
use crate::templates::_partials::post::Post;
use hyper::Uri;
use minijinja::context;
use serde::Deserialize;
use serde_json::json;
use tracing::warn;

pub mod _post_id;

#[derive(Debug, Deserialize)]
struct PostForm {
    profile_id: String,
    content: String,
}

pub async fn post(req: AuthedRequest<'_>) -> ServerResult {
    let req = req.into_text().await?;
    let form: PostForm = req.get_form_data()?;

    // This should be the currently logged into profile, probably
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
                is_owner: true
            };
            Ok(post)
        },
    )?;

    let body = req.render("_partials/post.html", context! { post })?;

    let post_to_federate = get_post(&req.db, &post_id.to_string(), &req.domain)?;
    let followers = query_map!(
        &req.db,
        Follower { inbox: String },
        "FROM followers LEFT JOIN known_actors USING (actor_id) WHERE profile_id = ?1",
        [ profile_id ]
    );

    // Notify followers
    tokio::spawn(async move {
        let create_activity = post_to_federate.into_create();
        let body = json!(create_activity);
        for follower in followers {
            let body = body.to_string();
            let inbox_uri: Uri = follower.inbox.parse().unwrap();
            let res = send_as(&inbox_uri, &req.data.current_profile, body).await;
            let res = match res {
                Ok(r) => r,
                Err(e) => { return warn!("Failed to send confirmation to {}: {:?}", inbox_uri, e) }
            };

            let code = res.status();
            let res_body = res.text().await;
            match res_body {
                Ok(body) => debug!("Received {} {}", code, body),
                Err(e) => warn!("Unable to reach {}: {:?}", inbox_uri, e)
            };

        }
    });

    Ok(send(body))
}

pub async fn delete(req: AuthedRequest<'_>) -> ServerResult {
    let post_param = req.get_url_param(2, "Missing post ID")?;
    debug!("Deleting post {}", post_param);
    req.db.execute("DELETE FROM posts WHERE post_id = ?1", [post_param])?;
    Ok(send("".to_owned()))
}
