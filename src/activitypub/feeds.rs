use hyper::Method;
use openssl::pkey::PKey;
use hyper::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use crate::activitypub::signature::get_signature_header;
use crate::server::error::{bad_gateway, ServerError};
use tracing::log::debug;
use hyper::header::{HeaderName, HeaderValue};
use hyper::header::DATE;
use hyper::HeaderMap;
use chrono::Utc;

static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

pub async fn get_remote_actor(domain: &str, feed_id: i64, private_key_pem: &str) -> Result<String, ServerError>{
    // Sig test stuff
    let date = Utc::now();

    let target = "https://indieweb.social/@alexpetros";
    let mut headers = HeaderMap::new();
    let date_header = HeaderValue::from_bytes(date.format("%a, %d %b %Y %X GMT").to_string().as_bytes())?;
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())?;
    let key_id = format!("https://{}/feeds/{}#main-key", &domain, feed_id);
    let signature = get_signature_header(Method::GET, &key_id, target, &domain, date, pkey)?;
    debug!("{}", signature);
    let signature = HeaderValue::from_str(&signature)?;

    headers.insert(USER_AGENT, HeaderValue::from_static("curl/8.4.0"));
    headers.insert(DATE, date_header);
    headers.insert(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER));
    headers.insert(HeaderName::from_static("signature"), signature);

    let client = reqwest::Client::new()
        .request(Method::GET, target)
        .headers(headers);

    let request = client.send().await
        .map_err( |e| {
            let message = format!("{:?}", e);
            bad_gateway(&message)
        })?;

    let req_body = request.text().await
        .map_err(|e| {
            let message = format!("{:?}", e);
            bad_gateway(&message)
        })?;

    Ok(req_body)
}
