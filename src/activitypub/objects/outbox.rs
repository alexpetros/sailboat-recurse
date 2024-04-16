use super::{AtContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderedCollectionType {
    OrderedCollection
}

// This is here to acknowledge that these could just be embedded JSON themselves
// Although Mastodon doesn't do that, so I'm going to punt on it for now and assume they're links
// type PageOrLink = String;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PageOrLink {
    Link(String),
    Page(OrderedCollectionPage)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderedCollectionPage {
    #[serde(rename = "@context")]
    pub context: AtContext,
    pub next: Option<Box<PageOrLink>>,
    pub prev: Option<Box<PageOrLink>>,
    #[serde(rename = "orderedItems")]
    pub ordered_items: Vec<Activity>
}
// https://www.w3.org/TR/activitystreams-core/#collection
#[derive(Debug, Serialize, Deserialize)]
pub struct Outbox {
    #[serde(rename = "@context")]
    pub context: AtContext,
    #[serde(rename = "type")]
    _type: OrderedCollectionType,
    pub id: String,
    #[serde(rename = "totalItems")]
    pub total_items: i64,
    pub first: PageOrLink,
    pub last: PageOrLink,
    pub current: Option<PageOrLink>, // This is theoretically mandatory
    pub items: Option<Vec<Value>>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActivityType {
    Create,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "@context")]
    pub context: Option<AtContext>,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub id: String,
    pub actor: Option<String>,
    pub published: Option<String>,
    pub to: Option<Vec<String>>,
    pub cc: Option<Vec<String>>,
    pub object: Object
}

// https://www.w3.org/TR/activitystreams-vocabulary/#object-types
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Object {
    Note(Note),
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NoteType { Note }

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    #[serde(rename = "type")]
    note_type: NoteType,
    pub summary: Option<String>,
    pub published: Option<String>,
    pub url: String,
    pub content: String
}
