use serde_json::json;
use crate::activitypub::LinkType;
use crate::activitypub::Link;
use crate::activitypub::WebFinger;
use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::response;
use crate::server::response::ResponseResult;

pub async fn get(_req: Request, _ctx: Context<'_>) -> ResponseResult {
    // let body = req.get_body().await?;
    // let text = body.text()?;

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
