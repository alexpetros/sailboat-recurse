use rusqlite::Connection;
use chrono::{DateTime, Local};
use serde::Serialize;
use hyper::Uri;
use tracing::warn;
use crate::activitypub::FullHandle;
use crate::activitypub::objects::actor::{Actor, LinkType};
use crate::activitypub::requests::{get_remote_actor, get_webfinger};
use crate::server::error::{map_bad_gateway, ServerError};

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

// TODO don't just use Actor 1, obviously
const PROFILE_ID: i64 = 1;

pub async fn get_or_search_for_actor(db: &mut Connection, domain: &str, handle: &FullHandle) -> Result<Option<Actor>, ServerError> {
    // First check if we are following them
    let FullHandle { preferred_username, host } = handle;
    // let actor = db.query_row("
    // SELECT name, url, preferred_username, host, summary
    // FROM followed_actors
    // WHERE handle = ? AND host = ?",
    // (preferred_username, host), |row| {
    //     let actor = Actor {
    //         name: row.get(0)?,
    //         url: row.get(1)?,
    //         preferred_username: row.get(2)?,
    //         host: row.get(3)?,
    //         summary: row.get(4)?,
    //         icon_url: "".to_owned(),
    //         is_followed: true
    //     };
    //     Ok(actor)
    // });

    // if actor.is_ok() { return Ok(actor.ok()); }

    let private_key_pem: String = db.query_row_and_then(
        "SELECT private_key_pem FROM profiles WHERE profile_id = ?1",
        [PROFILE_ID],
        |row| row.get(0))?;

    let web_finger = get_webfinger(host, preferred_username).await?;
    let self_link = web_finger.links
        .as_ref()
        .map(|links| links.iter().find(|l| l.rel == "self" && l.link_type == Some(LinkType::ActivityJson)))
        .flatten()
        .map(|link| link.href.clone())
        .flatten();

    let uri = match self_link.as_ref() {
        None => return Ok(None),
        Some(link) => link.clone().parse::<Uri>()
    }.map_err(|e| {
        warn!("Invalid URI provided for self by {}@{}: {:?}", preferred_username, host, &self_link);
        map_bad_gateway(e)
    })?;

    let actor = get_remote_actor(domain, PROFILE_ID, &uri, &private_key_pem).await?;


    // let actor = Actor {
    //     name: actor.name,
    //     url: local_url,
    //     preferred_username: actor.preferred_username,
    //     host: host.to_owned(),
    //     summary: actor.summary.unwrap_or("".to_owned()),
    //     icon_url,
    //     is_followed: false,
    //     outbox: actor.outbox.unwrap()
    // };

    Ok(Some(actor))
}
