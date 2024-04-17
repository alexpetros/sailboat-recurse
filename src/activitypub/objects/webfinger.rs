use crate::activitypub::objects::actor::LinkType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebFingerLink {
    pub rel: String,
    #[serde(rename = "type")]
    pub link_type: Option<LinkType>,
    pub href: Option<String>,
}

// Can we do a URI-formatted string property? Feels like yes
// https://datatracker.ietf.org/doc/html/rfc7033
#[derive(Debug, Serialize, Deserialize)]
pub struct WebFinger {
    pub subject: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub properties: Option<serde_json::Value>,
    pub links: Option<Vec<WebFingerLink>>,
}
