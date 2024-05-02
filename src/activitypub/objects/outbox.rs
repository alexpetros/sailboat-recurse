use crate::{activitypub::PUBLIC_STREAM, query_row_custom, server::server_response::InternalResult};

use super::{note::{Note, Post}, AtContext, Context};
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
    pub ordered_items: Vec<CreateActivity>,
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
    Accept,
    Follow,
    Create,
    Undo,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptActivity<A> {
    #[serde(rename = "@context")]
    pub context: AtContext,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub id: String,
    pub actor: String,
    pub object: A
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UndoActivity<A> {
    #[serde(rename = "@context")]
    pub context: Option<AtContext>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub actor: String,
    pub object: A
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FollowActivity {
    #[serde(rename = "@context")]
    pub context: Option<AtContext>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub actor: String,
    pub object: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateActivity {
    #[serde(rename = "@context")]
    pub context: Option<AtContext>,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub id: String,
    pub actor: String,
    pub published: Option<String>,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    pub object: Object,
}

impl From<Note> for CreateActivity {
    fn from(note: Note) -> Self {
        CreateActivity {
            context: Some(AtContext::Context(Context::ActivityStreams)),
            activity_type: ActivityType::Create,
            id: note.url.to_owned(),
            actor: note.attributed_to.to_owned(),
            published: note.published.to_owned(),
            to: vec![PUBLIC_STREAM.to_owned()],
            cc: vec![format!("{}/followers", note.attributed_to)],
            object: Object::Note(note)
        }
    }
}

impl Note {
    pub fn into_create(self: Note) -> CreateActivity {
        self.into()
    }
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
    let posts = db.query_row(
        "SELECT post_id, profile_id, content, created_at FROM posts WHERE profile_id = ?1",
        [ profile_id ],
        |row| {
            let post_id = row.get(0)?;
            let profile_id: i64 = row.get(1)?;
            let post = Post {
                post_id,
                content: row.get(2)?,
                created_at: row.get(3)?,
                url: format!("{}/posts/{}", domain, post_id),
                actor_id: format!("{}/profiles/{}", domain, profile_id)
            };
            Ok(post)
        });

    let page_url = format!("{}/profiles/{}/outbox?page=1", domain, profile_id);
    let items: Vec<CreateActivity>  = posts.into_iter()
        .map(|post| { post.into_note() })
        .map(|note| { note.into_create() })
        .collect();


    let page = OrderedCollectionPage {
        context: AtContext::Context(Context::ActivityStreams),
        next: None,
        prev: Some(Box::new(PageOrLink::Link(page_url.to_owned()))),
        ordered_items: items
    };

    Ok(page)
}
