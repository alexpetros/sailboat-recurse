use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
enum Context {
    #[serde(rename = "https://www.w3.org/ns/activitystreams")]
    ActivityStreams,
    #[serde(rename = "https://w3id.org/security/v1")]
    SecurityV1
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
