use openssl::pkey::PKey;
use serde::Serialize;
use serde::Deserialize;

pub mod signature;
pub mod feeds;

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
