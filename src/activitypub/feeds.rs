use chrono_tz::Etc::GMT;
use hyper::Method;
use openssl::pkey::PKey;
use hyper::header::ACCEPT;
use crate::activitypub::signature::get_signature_header;
use crate::server::error::{bad_gateway, ServerError};
use tracing::log::debug;
use hyper::header::{HeaderName, HeaderValue};
use hyper::header::DATE;
use hyper::HeaderMap;
use chrono::Utc;

static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

pub async fn get_remote_actor(_domain: &str, feed_id: i64, private_key_pem: &str) -> Result<String, ServerError>{
    // Sig test stuff
    let date = Utc::now().with_timezone(&GMT);

    let host = "indieweb.social";
    let target = "/@alexpetros";

    let mut headers = HeaderMap::new();
    let date_header = HeaderValue::from_bytes(date.format("%a, %d %b %Y %X %Z").to_string().as_bytes())?;
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())?;
    // let key_id = format!("https://{}/feeds/{}#main-key", &domain, feed_id);
    let key_id = format!("https://3a6d-2602-fb65-0-100-b46b-fc82-5adb-2e65.ngrok-free.app/feeds/{}#main-key", feed_id);
    let signature = get_signature_header(Method::GET, &key_id, target, host, date, pkey)?;
    debug!("{}", signature);
    let signature = HeaderValue::from_str(&signature)?;

    headers.insert(DATE, date_header);
    headers.insert(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER));
    headers.insert(HeaderName::from_static("signature"), signature);

    let client = reqwest::Client::new()
        .request(Method::GET, "https://indieweb.social/@alexpetros")
        .headers(headers);

    let request = client.send().await
        .map_err( |e| {
            let message = format!("{:?}", e);
            debug!("{}", message);
            bad_gateway(&message)
        })?;

    let req_body = request.text().await
        .map_err(|e| {
            let message = format!("{:?}", e);
            debug!("{}", message);
            bad_gateway(&message)
        })?;

    Ok(req_body)
}
