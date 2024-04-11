use rusqlite::Connection;
use chrono::{DateTime, Local};
use serde::Serialize;
use crate::server::error::ServerError;

#[derive(Debug, Serialize)]
pub struct Post {
    post_id: i64,
    display_name: String,
    handle: String,
    content: String,
    created_at: String
}

pub fn get_posts_in_profile (db: &Connection, profile_id: i64) -> Result<Vec<Post>, ServerError> {
    let mut query = db.prepare(
        "SELECT post_id,
            display_name,
            handle,
            content,
            created_at
         FROM posts
         LEFT JOIN profiles AS f USING (profile_id)
         WHERE profile_id = ?1
         ORDER BY created_at DESC
         ")?;
    let rows = query.query_map([profile_id], |row| {
        // TODO: User-local timestamp handling
        let created_at = DateTime::from_timestamp(row.get(4)?, 0)
            .unwrap()
            .with_timezone(&Local);
        let created_at = created_at.format("%b %m %Y, %r").to_string();
        let post = Post {
            post_id: row.get(0)?,
            display_name: row.get(1)?,
            handle: row.get(2)?,
            content: row.get(3)?,
            created_at
        };
        Ok(post)
    })?;

    let mut posts = Vec::new();
    for post in rows {
        posts.push(post?)
    }

    Ok(posts)
}
