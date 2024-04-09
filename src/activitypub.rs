use chrono::Utc;
use chrono_tz::Etc::GMT;
use hyper::header::{HeaderName, HeaderValue, USER_AGENT};
use hyper::header::ACCEPT;
use hyper::header::DATE;
use hyper::{Method, Uri};
use openssl::pkey;
use openssl::pkey::PKey;
use reqwest::RequestBuilder;
use serde::Deserialize;
use serde::Serialize;

use crate::server::error::{map_bad_gateway, ServerError};

use self::signature::get_signature_header;

pub mod signature;
pub mod actors;

static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

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
    pub links: Option<Vec<WebFingerLink>>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Context {
    #[serde(rename = "https://www.w3.org/ns/activitystreams")]
    ActivityStreams,
    #[serde(rename = "https://w3id.org/security/v1")]
    SecurityV1,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActorType {
    Person
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    #[serde(rename = "type")]
    pub icon_type: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String
}

impl PublicKey {
    pub fn new (id: &str, private_key_pem: &str) -> PublicKey {
        let private_key_pem = PKey::private_key_from_pem(private_key_pem.as_bytes()).unwrap();
        let public_key_pem = private_key_pem.public_key_to_pem().unwrap();
        PublicKey {
            id: format!("{}#main-key", id),
            owner: id.to_owned(),
            public_key_pem: String::from_utf8(public_key_pem).unwrap().trim().to_owned()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "@context")]
    pub context: Vec<Context>,
    pub id: String,
    pub url: String,
    pub summary: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub actor_type: ActorType,
    #[serde(rename = "preferredUsername")]
    pub preferred_username: String, // This might be optional
    pub inbox: String,
    pub outbox: String,
    #[serde(rename = "publicKey")]
    pub public_key: PublicKey,
    pub icon: Option<Icon>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    #[serde(rename = "application/activity+json")]
    ActivityJson,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

pub fn build_activitypub_request(method: Method, domain: &str, feed_id: i64, uri: &Uri, pkey: PKey<pkey::Private>) -> Result<RequestBuilder, ServerError> {
    let date = Utc::now().with_timezone(&GMT);
    let date_header = HeaderValue::from_bytes(date.format("%a, %d %b %Y %X %Z").to_string().as_bytes())?;
    let key_id = format!("https://{}/feeds/{}#main-key", &domain, feed_id);
    let signature = get_signature_header(&method, &key_id, &uri, date, pkey)?;

    let client = reqwest::Client::new();
    let url = uri.to_string();
    let request = client.request(method, url)
        .header(DATE, date_header)
        .header(USER_AGENT, "Mastodon/3.1.3")
        .header(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER))
        .header(HeaderName::from_static("signature"), signature);

    Ok(request)
}

pub async fn get_webfinger(host: &str, account_name: &str) -> Result<WebFinger, ServerError> {
    let uri = format!("https://{}/.well-known/webfinger", host);
    let resource = format!("acct:{}@{}", account_name, host);
    let request = reqwest::Client::new()
        .get(uri)
        .header(USER_AGENT, "curl/8.4.0")
        .header(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER))
        .query(&[("resource", resource)]);
    let res = request.send().await.map_err(map_bad_gateway)?;
    let text = res.text().await.map_err(map_bad_gateway)?;

    let web_finger: WebFinger = serde_json::from_str(&text).map_err(|e| {
        // warn!("Webfinger response {}", text);
        map_bad_gateway(e)
    })?;
    Ok(web_finger)
}
