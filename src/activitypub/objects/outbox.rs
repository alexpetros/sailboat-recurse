use crate::{activitypub::PUBLIC_STREAM, query_map, query_row_custom, server::server_response::InternalResult};

use super::{note::Note, AtContext, Context};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderedCollectionType {
    OrderedCollection,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PageOrLink {
    Link(String),
    Page(OrderedCollectionPage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderedCollectionPage {
    #[serde(rename = "@context")]
    pub context: AtContext,
    pub next: Option<Box<PageOrLink>>,
    pub prev: Option<Box<PageOrLink>>,
    #[serde(rename = "orderedItems")]
    pub ordered_items: Vec<Activity>,
}
pub type OutboxPage = OrderedCollectionPage;

// type OutboxPage = OrderedCollectionPage;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<PageOrLink>, // This is theoretically mandatory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Value>>,
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
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    pub object: Object,
}

// https://www.w3.org/TR/activitystreams-vocabulary/#object-types
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Object {
    Note(Note),
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

pub fn get_outbox(db: &Connection, profile_id: i64, domain: &str) -> InternalResult<Outbox> {
    let profile = query_row_custom!(
        db,
        Profile { total_items: i64 },
        "SELECT count(*) as total_items FROM posts WHERE profile_id = ?",
        [profile_id]
    )?;

    let outbox_url = format!("{}/profiles/{}/outbox", domain, profile_id);
    // TODO: Pagination
    let first_page = format!("{}/profiles/{}/outbox?page=1", domain, profile_id);
    let last_page = format!("{}/profiles/{}/outbox?page=1", domain, profile_id);

    let outbox = Outbox {
        context: AtContext::Context(Context::ActivityStreams),
        id: outbox_url,
        _type: OrderedCollectionType::OrderedCollection,
        total_items: profile.total_items,
        items: None,
        first: PageOrLink::Link(first_page),
        last: PageOrLink::Link(last_page),
        current: None
    };

    Ok(outbox)
}

pub fn get_outbox_page(db: &Connection, profile_id: i64, domain: &str, _page_num: usize) -> InternalResult<OutboxPage> {
    let posts = query_map!(
        db,
        Post {
            post_id: i64,
            profile_id: i64,
            created_at: String,
            content: String
        },
        "FROM posts WHERE profile_id = ?1",
        [ profile_id ]);

    let page_url = format!("{}/profiles/{}/outbox?page=1", domain, profile_id);

    let items = posts.into_iter().map(|post| {
        let post_url = format!("{}/posts/{}", domain, post.post_id);
        let to_field = vec![PUBLIC_STREAM.to_owned()];
        let cc_field = vec![];

        let note = Note {
            id: page_url.to_owned(),
            _type: super::note::NoteType::Note,
            url: post_url.to_owned(),
            summary: None,
            published: Some(post.created_at.to_owned()),
            attributed_to: Some(post_url.to_owned()),
            to: to_field.clone(),
            cc: cc_field.clone(),
            sensitive: false,
            content: post.content,
            tag: vec![]
        };
        Activity {
            context: Some(AtContext::Context(Context::ActivityStreams)),
            activity_type: ActivityType::Create,
            id: page_url.to_owned(),
            actor: Some(page_url.to_owned()),
            published: Some(post.created_at.to_owned()),
            to:to_field.clone(),
            cc: cc_field.clone(),
            object: Object::Note(note)
        }
    }).collect();


    let page = OrderedCollectionPage {
        context: AtContext::Context(Context::ActivityStreams),
        next: None,
        prev: Some(Box::new(PageOrLink::Link(page_url.to_owned()))),
        ordered_items: items
    };

    Ok(page)
}
