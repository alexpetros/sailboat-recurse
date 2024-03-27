use chrono::Utc;
use chrono_tz::Etc::GMT;
use hyper::header::{HeaderName, HeaderValue};
use hyper::header::ACCEPT;
use hyper::header::DATE;
use hyper::Method;
use openssl::pkey;
use openssl::pkey::PKey;
use reqwest::RequestBuilder;
use serde::Deserialize;
use serde::Serialize;

use crate::server::error::ServerError;

use self::signature::get_signature_header;

pub mod signature;
pub mod feeds;

static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

#[derive(Debug, Serialize, Deserialize)]
pub enum Context {
    #[serde(rename = "https://www.w3.org/ns/activitystreams")]
    ActivityStreams,
    #[serde(rename = "https://w3id.org/security/v1")]
    SecurityV1
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActorType {
    Person
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
    pub name: String,
    #[serde(rename = "type")]
    pub actor_type: ActorType,
    #[serde(rename = "preferredUsername")]
    pub preferred_username: String,
    pub inbox: String,
    pub outbox: String,
    #[serde(rename = "publicKey")]
    pub public_key: PublicKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LinkType {
    #[serde(rename = "application/activity+json")]
    ActivityJson
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub link_type: LinkType,
    pub href: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebFinger {
    pub subject: String,
    pub links: Vec<Link>,
}

pub fn build_activitypub_request(method: Method, host: &str, target: &str, pkey: PKey<pkey::Private>) -> Result<RequestBuilder, ServerError> {
    let date = Utc::now().with_timezone(&GMT);
    let date_header = HeaderValue::from_bytes(date.format("%a, %d %b %Y %X %Z").to_string().as_bytes())?;
    // let key_id = format!("https://{}/feeds/{}#main-key", &domain, feed_id);
    let key_id = format!("https://a9b9-2602-fb65-0-100-703d-1c5-cc0c-986a.ngrok-free.app/feeds/{}#main-key", 1);
    let signature = get_signature_header(&method, &key_id, target, host, date, pkey)?;

    let uri = format!("https://{}/{}", host, target);

    let client = reqwest::Client::new();
    let request = client.request(method, uri)
        .header(DATE, date_header)
        .header(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER))
        .header(HeaderName::from_static("signature"), signature);

    Ok(request)
}
