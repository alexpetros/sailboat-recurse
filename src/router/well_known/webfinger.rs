use tracing::debug;
use tracing::warn;
use crate::server::response;
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;
use crate::activitypub::LinkType;
use crate::activitypub::Link;
use crate::activitypub::WebFinger;
use crate::server::context::Context;
use crate::server::error::bad_request;
use crate::server::request::Request;
use crate::server::response::send_status;
use crate::server::response::ResponseResult;

#[derive(Debug, Deserialize)]
struct Feed { feed_id: i64, }

#[derive(Debug, Deserialize)]
struct Query {
    resource: String
}

pub async fn get(req: Request, ctx: Context<'_>) -> ResponseResult {
    let query = req.uri().query().ok_or(bad_request("Missing query parameter"))?;

    let resource = serde_html_form::from_str::<Query>(query)
        .map(|q| { q.resource })
        .map_err(|_| { bad_request("Invalid query string provided") })?;

    let ( search_type, identifier ) = resource
        .split_once(":")
        .ok_or_else(|| { bad_request("Invalid resource query provided (missing scheme i.e. 'acct:')") })?;

    if search_type != "acct" {
        warn!("Receieved search type: {}", search_type);
        return Err(bad_request("Sorry, that scheme is not supported yet (expected 'acct:')"));
    }

    let ( handle, domain ) = identifier
        .split_once("@")
        .ok_or_else(|| { bad_request("Invalid handle resource provided") })?;

    debug!("Searching for user {}", handle);

    let db_domain: String = ctx.db
        .query_row("SELECT value FROM globals WHERE key = 'domain'", (), |row| { row.get(0) })?;

    if domain != db_domain {
        return send_status(StatusCode::NOT_FOUND)
    }

    let feed = ctx.db.query_row("
        SELECT feed_id
        FROM feeds
        WHERE handle = ?1
    ", [ handle ], |row| {
        let feed = Feed { feed_id: row.get(0)? };
        Ok(feed)
    });

    let feed = match feed {
        Ok(x) => x,
        Err(_) => return send_status(StatusCode::NOT_FOUND)
    };

    let self_link = Link {
        rel: "self".to_owned(),
        link_type: LinkType::ActivityJson,
        href: format!("https://{}/feeds/{}", domain, feed.feed_id)
    };

    let mut links = Vec::new();
    links.push(self_link);

    let actor = WebFinger {
        subject: format!("acct:{}@{}", handle, domain),
        links
    };

    let body = json!(actor).to_string();
    Ok(response::send(body))
}
