use hyper::StatusCode;
use crate::server::response::{redirect, send_status};
use serde_json::json;
use serde::Deserialize;
use crate::activitypub::{self, Actor};

use crate::server::error::{self, bad_request};
use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::response::{ResponseResult, send};

pub mod new;

#[derive(Deserialize)]
struct Feed {
    feed_id: i64,
    handle: String
}

#[derive(Deserialize)]
struct NewFeed {
    handle: String,
    display_name: String,
    internal_name: String
}

pub fn get(req: Request, ctx: Context<'_>) -> ResponseResult {
    let feed_id = req.uri().path().split("/")
        .nth(2)
        .ok_or(error::bad_request("Missing feed ID"))?
        .parse::<i64>()
        .map_err(|_| { bad_request("Invalid feed ID") })?;

    let feed = ctx.db
        .query_row("SELECT feed_id, handle FROM feeds where feed_id = ?1", [ feed_id ], |row| {
            let feed = Feed {
                feed_id: row.get(0)?,
                handle: row.get(1)?
            };
            Ok(feed)
        });

    let feed = match feed {
        Ok(x) => x,
        Err(_) => return send_status(StatusCode::NOT_FOUND)
    };

    let domain: String = ctx.db
        .query_row("SELECT value FROM globals WHERE key = 'domain'", (), |row| { row.get(0) })?;

    let id = format!("https://{}/feeds/{}", domain, feed.feed_id);
    let inbox = format!("https://{}/inbox", domain);

    let mut context = Vec::new();
    context.push(activitypub::Context::ActivityStreams);
    context.push(activitypub::Context::SecurityV1);
    let actor = Actor {
        context,
        id: id.to_owned(),
        actor_type: activitypub::ActorType::Person,
        preferred_username: feed.handle,
        inbox,
        public_key: activitypub::PublicKey::new(&id)
    };

    let body = json!(actor).to_string();
    Ok(send(body))
}

pub async fn post(req: Request, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: NewFeed = serde_html_form::from_str(&text)?;
    ctx.db.execute(
        "INSERT INTO feeds (handle, display_name, internal_name) VALUES (?1, ?2, ?3)",
        (&form.handle, &form.display_name, &form.internal_name)
    )?;

    let id = ctx.db.last_insert_rowid();
    let path = format!("/feeds/{}", id);

    redirect(&path)
}
