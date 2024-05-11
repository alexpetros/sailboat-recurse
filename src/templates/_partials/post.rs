use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Post {
    pub post_id: i64,
    pub content: String,
    pub created_at: String,
    pub actor_name: String,
    pub actor_handle: String,
    pub is_owner: bool
}
