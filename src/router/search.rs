use hyper::Uri;
use minijinja::context;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::activitypub::requests::get_remote_actor;
use crate::activitypub::requests::get_webfinger;
use crate::activitypub::objects::actor::LinkType;
use crate::server::error::{bad_request, map_bad_gateway, ServerError};
use crate::server::server_response;
use crate::server::server_response::send;
use crate::server::server_response::ServerResponse;
use crate::server::server_request::IncomingRequest;

#[derive(Deserialize)]
struct Query {
    q: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Actor {
    name: String,
    url: String,
    handle: String,
    summary: String,
    host: String,
    icon_url: String,
    is_followed: bool,
}

// TODO don't just use Actor 1, obviously
const PROFILE_ID: i64 = 1;

pub fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let body = req.render("search.html", context! {});
    Ok(server_response::send(body))
}

pub async fn post(req: IncomingRequest<'_>) -> ServerResponse {
    let mut req = req.to_text().await?;
    let query: Query = req.get_form_data()?;
    let username = query.q;
    let mut splits = username.splitn(3, '@').peekable();

    splits.next_if_eq(&"");
    let handle = splits.next().ok_or(bad_request("Missing user name"))?.trim();
    let host = splits.next().ok_or(bad_request("Missing host name"))?.trim();
    let actor = get_or_search_for_actor(&mut req.db, &req.domain, &handle, &host).await?;
    let actor = match actor {
        None => return Ok(send("No account found")),
        Some(actor) => actor
    };

    let context = context!{ user => actor };

    Ok(send(req.render("user-search-result.html", context)))
}

async fn get_or_search_for_actor(db: &mut Connection, domain: &str, handle: &str, host: &str) -> Result<Option<Actor>, ServerError> {
    // First check if we are following them
    let actor = db.query_row(
    "SELECT name, url, handle, host, summary FROM followed_actors WHERE handle = ? AND host = ?",
    (handle, host), |row| {
        let actor = Actor {
            name: row.get(0)?,
            url: row.get(1)?,
            handle: row.get(2)?,
            host: row.get(3)?,
            summary: row.get(4)?,
            icon_url: "".to_owned(),
            is_followed: true
        };
        Ok(actor)
    });

    if actor.is_ok() { return Ok(actor.ok()); }

    let private_key_pem: String = db.query_row_and_then(
        "SELECT private_key_pem FROM profiles WHERE profile_id = ?1",
        [PROFILE_ID],
        |row| row.get(0))?;

    let web_finger = get_webfinger(host, handle).await?;
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
        warn!("Invalid URI provided for self by {}@{}: {:?}", handle, host, &self_link);
        map_bad_gateway(e)
    })?;

    let actor = get_remote_actor(domain, PROFILE_ID, &uri, &private_key_pem).await?;

    let icon_url = actor.icon.map(|i| i.url).unwrap_or("".to_owned());
    let actor = Actor {
        name: actor.name,
        url: actor.url,
        handle: actor.preferred_username,
        host: host.to_owned(),
        summary: actor.summary.unwrap_or("".to_owned()),
        icon_url,
        is_followed: false
    };

    Ok(Some(actor))
}
