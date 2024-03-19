use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
enum Context {
    ActivityStreams,
    SecurityV1
}

impl Context {
    fn as_str(&self) -> &'static str {
        match self {
            Context::ActivityStreams => "https://www.w3.org/ns/activitystreams",
            Context::SecurityV1 => "https://w3id.org/security/v1"
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum ActorType {
    Person
}

#[derive(Debug, Serialize, Deserialize)]
struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "@context")]
    pub context: Vec<Context>,
    pub id: String,
    pub actor_type: ActorType,
    #[serde(rename = "preferredUsername")]
    pub preferred_username: String,
    pub inbox: String,
    #[serde(rename = "publicKey")]
    pub public_key: PublicKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LinkType {
    ActivityJson
}

impl LinkType {
    fn as_str(&self) -> &'static str {
        match self {
            LinkType::ActivityJson => "application/activity+json"
        }
    }
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
