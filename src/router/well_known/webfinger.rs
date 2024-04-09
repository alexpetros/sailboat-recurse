use tracing::debug;
use tracing::warn;
use crate::activitypub::LinkType;
use crate::server::response;
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;
use crate::activitypub::WebFinger;
use crate::activitypub::WebFingerLink;
use crate::server::error::bad_request;
use crate::server::request::IncomingRequest;
use crate::server::response::send_status;
use crate::server::response::ResponseResult;

#[derive(Debug, Deserialize)]
struct Feed { feed_id: i64, }

#[derive(Debug, Deserialize)]
struct Query {
    resource: String
}

pub async fn get(req: IncomingRequest<'_>) -> ResponseResult {
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

    if domain != req.domain {
        return send_status(StatusCode::NOT_FOUND)
    }

    let feed = req.db.query_row("
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

    let self_link = WebFingerLink {
        rel: "self".to_owned(),
        link_type: Some(LinkType::ActivityJson),
        href: Some(format!("https://{}/feeds/{}", domain, feed.feed_id))
    };

    let mut links = Vec::new();
    links.push(self_link);

    let actor = WebFinger {
        subject: Some(format!("acct:{}@{}", handle, domain)),
        aliases: None,
        properties: None,
        links: Some(links)
    };

    let body = json!(actor).to_string();
    Ok(response::send(body))
}
