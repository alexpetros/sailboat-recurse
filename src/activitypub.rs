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
    id: String,
    owner: String,
    public_key_pem: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "@context")]
    context: Vec<Context>,
    id: String,
    actor_type: ActorType,
    #[serde(rename = "preferredUsername")]
    preferred_username: String,
    inbox: String,
    #[serde(rename = "publicKey")]
    public_key: PublicKey,
}
