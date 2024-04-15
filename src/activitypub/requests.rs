use hyper::{Method, Uri};
use openssl::pkey::PKey;
use hyper::header::{ACCEPT, DATE, HeaderName, HeaderValue, USER_AGENT};
use openssl::pkey;
use reqwest::RequestBuilder;
use chrono::Utc;
use chrono_tz::Etc::GMT;
use crate::activitypub::SHORT_ACCEPT_HEADER;
use crate::activitypub::objects::actor::Actor;
use crate::activitypub::objects::webfinger::WebFinger;
use crate::activitypub::signature::get_signature_header;
use crate::server::error::{map_bad_gateway, ServerError};
use crate::server::utils;

fn build_activitypub_request(method: Method, domain: &str, profile_id: i64, uri: &Uri, pkey: PKey<pkey::Private>) -> Result<RequestBuilder, ServerError> {
    let date = Utc::now().with_timezone(&GMT);
    let date_header = HeaderValue::from_bytes(date.format("%a, %d %b %Y %X %Z").to_string().as_bytes())?;
    let key_id = format!("https://{}/profiles/{}#main-key", &domain, profile_id);
    let signature = get_signature_header(&method, &key_id, &uri, date, pkey)?;

    let client = reqwest::Client::new();
    let url = uri.to_string();
    let request = client.request(method, url)
        .header(DATE, date_header)
        .header(USER_AGENT, "Mastodon/3.1.3")
        .header(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER))
        .header(HeaderName::from_static("signature"), signature);

    Ok(request)
}


// TODO this could probably be a "Valid activitypub URI type"
pub async fn get_remote_actor(domain: &str, profile_id: i64, uri: &Uri, private_key_pem: &str) -> Result<Actor, ServerError> {
    // Sig test stuff
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())?;

    let request = build_activitypub_request(Method::GET, domain, profile_id, uri, pkey)?;
    let res = request.send().await.map_err(map_bad_gateway)?;

    let body = res.text().await.map_err(map_bad_gateway)?;
    let actor: Actor = utils::deserialize_json(&body)?;

    Ok(actor)
}

pub async fn get_webfinger(host: &str, account_name: &str) -> Result<WebFinger, ServerError> {
    let uri = format!("https://{}/.well-known/webfinger", host);
    let resource = format!("acct:{}@{}", account_name, host);
    let request = reqwest::Client::new()
        .get(uri)
        .header(USER_AGENT, "curl/8.4.0")
        .header(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER))
        .query(&[("resource", resource)]);
    let res = request.send().await.map_err(map_bad_gateway)?;
    let text = res.text().await.map_err(map_bad_gateway)?;

    let web_finger: WebFinger = utils::deserialize_json(&text)?;
    Ok(web_finger)
}

