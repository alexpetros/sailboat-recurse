use crate::activitypub::objects::Context;
use minijinja::Value;
use openssl::pkey::PKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ActorType {
    Person,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    #[serde(rename = "type")]
    pub icon_type: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}

impl PublicKey {
    pub fn new(id: &str, private_key_pem: &str) -> PublicKey {
        let private_key_pem = PKey::private_key_from_pem(private_key_pem.as_bytes()).unwrap();
        let public_key_pem = private_key_pem.public_key_to_pem().unwrap();
        PublicKey {
            id: format!("{}#main-key", id),
            owner: id.to_owned(),
            public_key_pem: String::from_utf8(public_key_pem).unwrap().trim().to_owned(),
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
    pub icon: Option<Icon>,
}

// TODO seems like minijinja could figure this out on its own via the serde::Serialize derivation
// Maybe try and PR that?
impl From<Actor> for minijinja::Value {
    fn from(value: Actor) -> Self {
        Value::from_serialize(value)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    #[serde(rename = "application/activity+json")]
    ActivityJson,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}
