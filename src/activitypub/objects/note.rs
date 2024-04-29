use minijinja::Value;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{activitypub::PUBLIC_STREAM, query_row_custom, server::server_response::InternalResult};

#[derive(Debug, Serialize, Deserialize)]
pub enum NoteType {
    Note
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub _type: NoteType,
    pub url: String,
    pub summary: Option<String>,
    pub published: Option<String>,
    #[serde(rename = "attributedTo")]
    pub attributed_to: Option<String>,
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

pub fn get_post(db: &Connection, post_id: &str, domain: &str) -> InternalResult<Note>{
    let post = query_row_custom!(
        db,
        Post {
            post_id: i64,
            profile_id: i64,
            content: String,
            created_at: String
        },
        "
        SELECT
            post_id,
            profile_id,
            content,
            created_at
        FROM posts
        LEFT JOIN profiles USING (profile_id)
        WHERE post_id = ?1
        ",
        [post_id])?;

    let post_url = format!("{}/posts/{}", domain, post_id);
    let actor_url = format!("{}/profiles/{}", domain, post.profile_id);

    let note = Note {
        id: post.post_id.to_string(),
        _type: NoteType::Note,
        url: post_url,
        summary: None,
        published: Some(post.created_at),
        attributed_to: Some(actor_url),
        to: vec![PUBLIC_STREAM.to_owned()],
        cc: vec![],
        sensitive: false,
        content: post.content,
        tag: vec![]
    };

    Ok(note)
}
