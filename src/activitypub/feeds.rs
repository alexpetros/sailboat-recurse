use hyper::Method;
use openssl::pkey::PKey;

use crate::activitypub::ServerError;
use crate::server::error::bad_gateway;

use super::build_activitypub_request;

pub async fn get_remote_actor(_domain: &str, _feed_id: i64, private_key_pem: &str) -> Result<String, ServerError>{
    // Sig test stuff
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())?;

    let host = "indieweb.social";
    let target = "/@alexpetros";

    let request = build_activitypub_request(Method::GET, host, target, pkey)?;
    let res = request.send().await.map_err(|e| {
        let message = format!("{}", e);
        bad_gateway(&message)
    })?;

    let body = res.text().await.map_err(|e| {
        let message = format!("{}", e);
        bad_gateway(&message)
    })?;

    Ok(body)
}
