use serde::{Deserialize, Serialize};

pub mod actor;
pub mod outbox;
pub mod webfinger;
pub mod note;

#[derive(Debug, Serialize, Deserialize)]
pub enum Context {
    #[serde(rename = "https://www.w3.org/ns/activitystreams")]
    ActivityStreams,
    #[serde(rename = "https://w3id.org/security/v1")]
    SecurityV1,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AtContext {
    Context(Context),
    Collection(Vec<Context>),
}
