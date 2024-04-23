use crate::activitypub::objects::actor::LinkType;
use crate::activitypub::objects::webfinger::WebFinger;
use crate::activitypub::objects::webfinger::WebFingerLink;
use crate::query_row;
use crate::server::error::bad_gateway;
use crate::server::error::bad_request;
use crate::server::error::map_bad_gateway;
use crate::server::error::not_found;
use crate::server::server_request::PlainRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::Error::SqliteFailure;
use serde::Deserialize;
use serde_json::json;
use tracing::debug;
use tracing::warn;

#[derive(Debug, Deserialize)]
struct Query {
    resource: String,
}

pub async fn get(req: PlainRequest<'_>) -> ServerResponse {
    let query = req
        .uri()
        .query()
        .ok_or(bad_request("Missing query parameter"))?;

    let resource = serde_html_form::from_str::<Query>(query)
        .map(|q| q.resource)
        .map_err(|_| bad_request("Invalid query string provided"))?;

    let (search_type, identifier) = resource.split_once(":").ok_or_else(|| {
        bad_request("Invalid resource query provided (missing scheme i.e. 'acct:')")
    })?;

    if search_type != "acct" {
        warn!("Received search type: {}", search_type);
        return Err(bad_request(
            "Sorry, that scheme is not supported yet (expected 'acct:')",
        ));
    }

    // TODO check the domain
    let (handle, domain) = identifier
        .split_once("@")
        .ok_or_else(|| bad_request("Invalid handle resource provided"))?;

    debug!("Searching for user {} {}", handle, domain);

    let profile = query_row!(
        req.db,
        Profile { profile_id: i64 },
        "FROM profiles where preferred_username = ?1",
        [handle]
        );


    println!("{:?}", profile);
    let profile = match profile {
        Ok(x) => Ok(x),
        Err(QueryReturnedNoRows) => Err(not_found()),
        Err(SqliteFailure(_, Some(m))) => Err(bad_gateway(&m)),
        Err(e) => Err(map_bad_gateway(e)),
    }?;


    let self_link = WebFingerLink {
        rel: "self".to_owned(),
        link_type: Some(LinkType::ActivityJson),
        href: Some(format!(
            "https://{}/profiles/{}",
            domain, profile.profile_id
        )),
    };

    let mut links = Vec::new();
    links.push(self_link);

    let actor = WebFinger {
        subject: Some(format!("acct:{}@{}", handle, domain)),
        aliases: None,
        properties: None,
        links: Some(links),
    };

    let body = json!(actor).to_string();
    Ok(server_response::send(body))
}
