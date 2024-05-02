use minijinja::Value;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{activitypub::PUBLIC_STREAM, server::server_response::InternalResult};

use super::outbox::CreateActivity;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub content: String,
    pub created_at: String,
    pub url: String,
    pub actor_id: String
}

impl Post {
    pub fn into_note(self: Post) -> Note {
        self.into()
    }

    pub fn into_create(self: Post) -> CreateActivity {
        self.into_note().into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NoteType {
    Note
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: NoteType,
    pub url: String,
    pub summary: Option<String>,
    pub published: Option<String>,
    #[serde(rename = "attributedTo")]
    pub attributed_to: String,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    #[serde(default)] // default false
    pub sensitive: bool,
    pub content: String,
    #[serde(default)]
    pub tag: Vec<String>
}

impl From<Note> for minijinja::Value {
    fn from(value: Note) -> Self {
        Value::from_serialize(value)
    }
}

impl From<Post> for Note {
    fn from(post: Post) -> Self {
        let cc = vec![format!("{}/followers", &post.actor_id)];
        Note {
            id: post.url.to_owned(),
            _type: NoteType::Note,
            url: post.url,
            summary: None,
            published: Some(post.created_at),
            attributed_to: post.actor_id,
            to: vec![PUBLIC_STREAM.to_owned()],
            cc,
            sensitive: false,
            content: post.content,
            tag: vec![]
        }
    }
}

pub fn get_post(db: &Connection, post_id: &str, domain: &str) -> InternalResult<Post>{
    let post = db.query_row(
        "
        SELECT post_id, profile_id, content, created_at
        FROM posts
        LEFT JOIN profiles USING (profile_id)
        WHERE post_id = ?1
        ",
        [post_id],
        |row| {
            let post_id = row.get(0)?;
            let profile_id: i64 = row.get(1)?;
            let post = Post {
                post_id,
                content: row.get(2)?,
                created_at: row.get(3)?,
                url: format!("https://{}/posts/{}", domain, post_id),
                actor_id: format!("https://{}/profiles/{}", domain, profile_id)
            };
            Ok(post)
        })?;

    Ok(post)
}
