use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Post {
    pub post_id: i64,
    pub actor_name: String,
    pub actor_handle: String,
    pub content: String,
    pub created_at: String
}
