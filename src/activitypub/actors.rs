use hyper::{Method, Uri};
use openssl::pkey::PKey;
use tracing::error;

use crate::{activitypub::ServerError, server::error::map_bad_gateway};

use super::{build_activitypub_request, Actor};

// TODO this could probably be a "Valid activitypub URI type"
pub async fn get_remote_actor(domain: &str, profile_id: i64, uri: &Uri, private_key_pem: &str) -> Result<Actor, ServerError> {
    // Sig test stuff
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())?;

    println!("Requesting {}", uri.to_string());
    let request = build_activitypub_request(Method::GET, domain, profile_id, uri, pkey)?;
    let res = request.send().await.map_err(map_bad_gateway)?;

    let body = res.text().await.map_err(map_bad_gateway)?;
    let actor: Actor = serde_json::from_str(&body).map_err(|e| {
        error!("Failed to deserialize body {}", body);
        map_bad_gateway(e)
    })?;

    Ok(actor)
}
