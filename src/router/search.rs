use hyper::Uri;
use minijinja::context;
use serde::Deserialize;
use tracing::warn;

use crate::activitypub::actors::get_remote_actor;
use crate::activitypub::{get_webfinger, LinkType};
use crate::server::context::Context;
use crate::server::error::{bad_request, map_bad_gateway};
use crate::server::response::send;
use crate::server::{request::IncomingRequest, response::ResponseResult};

#[derive(Deserialize)]
struct Query {
    q: String
}

// TODO don't just Actor 1, obviously
const FEED_ID: i64 = 1;

pub async fn post(req: IncomingRequest, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;

    let query = serde_html_form::from_str::<Query>(&text)?.q;
    let mut splits = query.splitn(3, '@').peekable();

    splits.next_if_eq(&"");
    let handle = splits.next().ok_or(bad_request("Missing user name"))?.trim();
    let host = splits.next().ok_or(bad_request("Missing host name"))?.trim();

    let private_key_pem: String = ctx.db.query_row_and_then(
        "SELECT private_key_pem FROM feeds WHERE feed_id = ?1",
        [FEED_ID],
        |row| row.get(0))?;

    let web_finger = get_webfinger(host, handle).await?;
    println!("{:?}", &web_finger);
    let self_link = web_finger.links
        .as_ref()
        .map(|links| links.iter().find(|l| l.rel == "self" && l.link_type == Some(LinkType::ActivityJson)))
        .flatten()
        .map(|link| link.href.clone())
        .flatten();

    let uri = match self_link.as_ref() {
        None => return Ok(send("No account found")),
        Some(link) => link.clone().parse::<Uri>()
    }.map_err(|e| {
        warn!("Invalid URI provided for self by {}@{}: {:?}", handle, host, &self_link);
        map_bad_gateway(e)
    })?;

    let actor = get_remote_actor(&req.domain, FEED_ID, &uri, &private_key_pem).await?;
    let icon_url = actor.icon.map(|i| i.url).unwrap_or("".to_owned());

    let user = context! {
        icon_url,
        name => actor.name,
        url => actor.url,
        handle => actor.preferred_username,
        summary => actor.summary.unwrap_or("".to_owned())
    };
    let context = context!{ user };

    Ok(send(ctx.render("user-search-result.html", context)))
}
