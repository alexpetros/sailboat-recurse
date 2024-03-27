use hyper::StatusCode;
use crate::server::response::send_status;
use hyper::header::HeaderValue;
use hyper::header::CONTENT_TYPE;
use tracing::log::debug;
use crate::activitypub::actors::get_remote_actor;
use serde::Deserialize;
use serde::Serialize;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use hyper::header::ACCEPT;
use minijinja::context;
use crate::queries::get_posts_in_feed;
use crate::server::response::{self, redirect};
use serde_json::json;
use crate::activitypub::{self, Actor};


use crate::server::error::{self, bad_request};
use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::response::{ResponseResult, send};

pub mod new;

#[derive(Serialize, Deserialize)]
struct Feed {
    feed_id: i64,
    handle: String,
    display_name: String,
    internal_name: String,
    private_key_pem: String
}

#[derive(Deserialize)]
struct NewFeed {
    handle: String,
    display_name: String,
    internal_name: String
}

static LONG_ACCEPT_HEADER: &str = "application/ld+json;profile=“https://www.w3.org/ns/activitystreams";
static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

pub async fn get(req: Request, ctx: Context<'_>) -> ResponseResult {
    let feed_param = req.uri().path().split("/")
        .nth(2)
        .ok_or(error::bad_request("Missing feed ID"))?;

    let feed_id = match feed_param.split_once("#") {
        None => feed_param,
        Some((f, _)) => f
    }.parse::<i64>().map_err(|_| { bad_request("Invalid feed ID") })?;

    let feed = ctx.db
        .query_row("
        SELECT feed_id, handle, display_name, internal_name, private_key_pem
        FROM feeds where feed_id = ?1"
        , [ feed_id ], |row| {
            let feed = Feed {
                feed_id: row.get(0)?,
                handle: row.get(1)?,
                display_name: row.get(2)?,
                internal_name: row.get(3)?,
                private_key_pem: row.get(4)?
            };
            Ok(feed)
        });

    let feed = match feed {
        Ok(x) => x,
        Err(_) => return send_status(StatusCode::NOT_FOUND)
    };

    // If no request header was provided, serve the HTML feed
    let request_header = req.headers().get(ACCEPT);

    // If a valid request header was provided and contains the correct accept value,
    // serve the JSON representation of the feed
    // TODO actually parse the header properly
    match request_header {
        None => serve_html_feed(req, ctx, feed).await,
        Some(h) => {
            let h = h.to_str().unwrap_or("");
            if h.contains(LONG_ACCEPT_HEADER) || h.contains(SHORT_ACCEPT_HEADER) {
                debug!("Found JSON header, Serving json feed");
                serve_json_feed(req, ctx, feed)
            } else {
                serve_html_feed(req, ctx, feed).await
            }
        }
    }
}

pub async fn post(req: Request, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: NewFeed = serde_html_form::from_str(&text)?;

    // TODO encrypt this
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap().private_key_to_pem_pkcs8().unwrap();
    let pkey = String::from_utf8(pkey).unwrap();

    ctx.db.execute(
        "INSERT INTO feeds (handle, display_name, internal_name, private_key_pem)
        VALUES (?1, ?2, ?3, ?4)",
        (&form.handle, &form.display_name, &form.internal_name, &pkey)
    )?;

    let id = ctx.db.last_insert_rowid();
    let path = format!("/feeds/{}", id);

    redirect(&path)
}

async fn serve_html_feed(_req: Request, ctx: Context<'_>, feed: Feed) -> ResponseResult {
    let domain: String = ctx.db
        .query_row("SELECT value FROM globals WHERE key = 'domain'", (), |row| { row.get(0) })?;
    let posts = get_posts_in_feed(&ctx.db, feed.feed_id)?;
    let context = context! { feed => feed, posts => posts };

    let req_body = get_remote_actor(&domain, feed.feed_id, &feed.private_key_pem)
        .await
        .unwrap_or_else(|e| {  e.message });
    let context = context! { req_body => req_body, ..context };

    let body = ctx.render("feed.html", context);
    Ok(response::send(body))
}

fn serve_json_feed(_req: Request, ctx: Context<'_>, feed: Feed) -> ResponseResult {
    let domain: String = ctx.db
        .query_row("SELECT value FROM globals WHERE key = 'domain'", (), |row| { row.get(0) })?;

    let id = format!("https://{}/feeds/{}", domain, feed.feed_id);
    let inbox = format!("https://{}/inbox", domain);
    let outbox = format!("https://{}/feeds/{}/outbox", domain, feed.feed_id);
    let public_key = activitypub::PublicKey::new(&id, &feed.private_key_pem);


    let mut context = Vec::new();
    context.push(activitypub::Context::ActivityStreams);
    context.push(activitypub::Context::SecurityV1);
    let actor = Actor {
        context,
        id: id.to_owned(),
        name: feed.handle.to_owned(),
        actor_type: activitypub::ActorType::Person,
        preferred_username: feed.handle,
        inbox,
        outbox,
        public_key
    };

    let body = json!(actor).to_string();
    let mut res = send(body);
    let header_value = HeaderValue::from_static("application/activity+json");
    res.headers_mut().append(CONTENT_TYPE, header_value);
    Ok(res)
}
