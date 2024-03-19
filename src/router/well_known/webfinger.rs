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
struct Query {
    resource: String
}

pub async fn get(req: Request, _ctx: Context<'_>) -> ResponseResult {
    let query = req.uri().query().ok_or(bad_request("Missing query parameter"))?;

    let user = serde_html_form::from_str::<Query>(query)
        .map(|q| { q.resource })?;

    if user != "acc:awp@example.com" {
        return send_status(StatusCode::NOT_FOUND);
    }

    let self_link = Link {
        rel: "self".to_owned(),
        link_type: LinkType::ActivityJson,
        href: "https://example.com/feeds/1".to_owned()
    };

    let mut links = Vec::new();
    links.push(self_link);

    let actor = WebFinger {
        subject: "acct:alex@example.com".to_owned(),
        links
    };

    let body = json!(actor).to_string();
    Ok(response::send(body))
}
