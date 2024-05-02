use crate::activitypub::objects::actor::Actor;
use crate::activitypub::objects::outbox::{OrderedCollectionPage, Outbox};
use crate::activitypub::objects::webfinger::WebFinger;
use crate::activitypub::signature::get_signature_header;
use crate::server::error::{map_bad_gateway, ServerError};
use crate::server::server_request::{CurrentProfile, SHORT_ACCEPT_HEADER};
use crate::server::server_response::InternalResult;
use crate::server::utils;
use chrono::Utc;
use chrono_tz::Etc::GMT;
use hyper::header::{HeaderName, HeaderValue, ACCEPT, DATE, USER_AGENT};
use hyper::{Method, Uri};
use openssl::base64;
use reqwest::{RequestBuilder, Response};
use serde::de::DeserializeOwned;

fn build_activitypub_request(
    method: Method,
    uri: &Uri,
    current_profile: &CurrentProfile,
    body: Option<String>
) -> Result<RequestBuilder, ServerError> {
    let CurrentProfile {
        domain,
        profile_id,
        pkey,
    } = current_profile;
    let date = Utc::now().with_timezone(&GMT);
    let date_header =
        HeaderValue::from_bytes(date.format("%a, %d %b %Y %X %Z").to_string().as_bytes())?;
    let key_id = format!("https://{}/profiles/{}#main-key", &domain, profile_id);

    let client = reqwest::Client::new();
    let url = uri.to_string();
    let mut request = client
        .request(method.clone(), url)
        .header(DATE, date_header)
        .header(USER_AGENT, "Mastodon/3.1.3")
        .header(ACCEPT, HeaderValue::from_static(SHORT_ACCEPT_HEADER));

    let signature;
    if let Some(body) = body {
        let vec = openssl::sha::sha256(body.as_bytes());
        let digest = base64::encode_block(&vec);
        let digest = format!("sha-256={}", digest);
        let header_name = HeaderName::from_lowercase(b"digest").unwrap();
        let header_value = HeaderValue::from_str(&digest)?;
        signature = get_signature_header(&method, &key_id, uri, date, pkey, Some(digest))?;
        request = request.header(header_name, header_value).body(body);
    } else {
        signature = get_signature_header(&method, &key_id, uri, date, pkey, None)?;
    }

    let request = request.header(HeaderName::from_static("signature"), signature);

    Ok(request)
}

async fn get_from_ap<'a, T>(uri: &Uri, current_profile: &CurrentProfile) -> InternalResult<T>
where
    T: DeserializeOwned,
{
    let request = build_activitypub_request(Method::GET, uri, current_profile, None)?;
    let res = request.send().await.map_err(map_bad_gateway)?;
    let body = res.text().await.map_err(map_bad_gateway)?;
    let item: T = utils::deserialize_json(&body)?;
    Ok(item)
}

pub async fn send_as<'a>(uri: &Uri, profile: &CurrentProfile, body: String) -> InternalResult<Response>
{
    let request = build_activitypub_request(Method::POST, uri, profile, Some(body))?;
    let res = request.send().await.map_err(map_bad_gateway)?;
    Ok(res)
}

pub async fn get_actor(uri: &Uri, current_profile: &CurrentProfile) -> InternalResult<Actor> {
    get_from_ap(uri, current_profile).await
}

pub async fn get_outbox(uri: &Uri, current_profile: &CurrentProfile) -> InternalResult<Outbox> {
    get_from_ap(uri, current_profile).await
}

pub async fn get_outbox_page(uri: &Uri, current_profile: &CurrentProfile) -> InternalResult<OrderedCollectionPage> {
    get_from_ap(uri, current_profile).await
}

pub async fn get_webfinger(host: &str, account_name: &str) -> InternalResult<WebFinger> {
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
