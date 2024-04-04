use serde::Deserialize;

use crate::activitypub::actors::get_remote_actor;
use crate::server::context::Context;
use crate::server::error::bad_request;
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
    let target = splits.next().ok_or(bad_request("Missing user name"))?.trim();
    let host = splits.next().ok_or(bad_request("Missing host name"))?.trim();

    let private_key_pem: String = ctx.db.query_row_and_then(
        "SELECT private_key_pem FROM feeds WHERE feed_id = ?1",
        [FEED_ID],
        |row| row.get(0))?;

    let actor = get_remote_actor(&req.domain, FEED_ID, &host, &target, &private_key_pem).await?;

    Ok(send(actor))
}
