use serde::Serialize;
use crate::server::context::Context;
use crate::server::error::ServerError;

#[derive(Debug, Serialize)]
pub struct Post {
    post_id: i64,
    author_name: String,
    author_handle: String,
    content: String
}

pub fn get_posts_in_feed (ctx: &Context<'_>) -> Result<Vec<Post>, ServerError> {
    let mut query = ctx.db.prepare("SELECT post_id, author_name, author_handle, content FROM posts")?;
    let rows = query.query_map((), |row| {
        let post = Post {
            post_id: row.get(0)?,
            author_name: row.get(1)?,
            author_handle: row.get(2)?,
            content: row.get(3)?
        };
        Ok(post)
    })?;

    let mut posts = Vec::new();
    for post in rows {
        posts.push(post?)
    }

    Ok(posts)
}
