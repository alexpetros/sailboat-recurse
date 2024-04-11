use hyper::StatusCode;
use crate::server::server_response::send_status;
use hyper::header::HeaderValue;
use hyper::header::CONTENT_TYPE;
use tracing::log::debug;
use serde::Deserialize;
use serde::Serialize;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use hyper::header::ACCEPT;
use minijinja::context;
use crate::queries::get_posts_in_profile;
use crate::server::server_response::{self, redirect};
use serde_json::json;
use crate::activitypub::objects::actor::{Actor, ActorType, Context, PublicKey};

use crate::server::error::{self, bad_request};
use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{send, ServerResponse};

pub mod new;

#[derive(Serialize, Deserialize)]
struct Profile {
    profile_id: i64,
    handle: String,
    display_name: String,
    internal_name: String,
    private_key_pem: String
}

#[derive(Deserialize)]
struct NewProfile {
    handle: String,
    display_name: String,
    internal_name: String
}

static LONG_ACCEPT_HEADER: &str = "application/ld+json;profile=“https://www.w3.org/ns/activitystreams";
static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

pub async fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let profile_param = req.uri().path().split("/")
        .nth(2)
        .ok_or(error::bad_request("Missing profile ID"))?;

    let profile_id = match profile_param.split_once("#") {
        None => profile_param,
        Some((f, _)) => f
    }.parse::<i64>().map_err(|_| { bad_request("Invalid profile ID") })?;

    let profile = req.db
        .query_row("
        SELECT profile_id, handle, display_name, internal_name, private_key_pem
        FROM profiles where profile_id = ?1"
        , [ profile_id ], |row| {
            let profile = Profile {
                profile_id: row.get(0)?,
                handle: row.get(1)?,
                display_name: row.get(2)?,
                internal_name: row.get(3)?,
                private_key_pem: row.get(4)?
            };
            Ok(profile)
        });

    let profile = match profile {
        Ok(x) => x,
        Err(_) => return send_status(StatusCode::NOT_FOUND)
    };

    // If no request header was provided, serve the HTML profile
    let request_header = req.headers().get(ACCEPT);

    // If a valid request header was provided and contains the correct accept value,
    // serve the JSON representation of the profile
    // TODO actually parse the header properly
    match request_header {
        None => serve_html_profile(req, profile).await,
        Some(h) => {
            let h = h.to_str().unwrap_or("");
            if h.contains(LONG_ACCEPT_HEADER) || h.contains(SHORT_ACCEPT_HEADER) {
                debug!("Found JSON header, Serving json profile");
                serve_json_profile(req, profile)
            } else {
                serve_html_profile(req, profile).await
            }
        }
    }
}

pub async fn post(req: IncomingRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
    let form: NewProfile = req.get_form_data()?;

    // TODO encrypt this
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap().private_key_to_pem_pkcs8().unwrap();
    let pkey = String::from_utf8(pkey).unwrap();

    req.db.execute(
        "INSERT INTO profiles (handle, display_name, internal_name, private_key_pem)
        VALUES (?1, ?2, ?3, ?4)",
        (&form.handle, &form.display_name, &form.internal_name, &pkey)
    )?;

    let id = req.db.last_insert_rowid();
    let path = format!("/profiles/{}", id);

    redirect(&path)
}

async fn serve_html_profile(req: IncomingRequest<'_>, profile: Profile) -> ServerResponse {
    // let domain = req.domain;
    let posts = get_posts_in_profile(&req.db, profile.profile_id)?;
    let context = context! { profile => profile, posts => posts };

    // let req_body = get_remote_actor(&domain, profile.profile_id, &profile.private_key_pem)
    //     .await
    //     .unwrap_or_else(|e| {  e.message });
    // let context = context! { req_body => req_body, ..context };

    let body = req.render("profile.html", context);
    Ok(server_response::send(body))
}

fn serve_json_profile(req: IncomingRequest<'_>, profile: Profile) -> ServerResponse {
    let domain = &req.domain;

    let id = format!("https://{}/profiles/{}", domain, profile.profile_id);
    let inbox = format!("https://{}/inbox", domain);
    let outbox = format!("https://{}/profiles/{}/outbox", domain, profile.profile_id);
    let public_key = PublicKey::new(&id, &profile.private_key_pem);

    let mut context = Vec::new();
    context.push(Context::ActivityStreams);
    context.push(Context::SecurityV1);
    let actor = Actor {
        context,
        id: id.to_owned(),
        url: id.to_owned(),
        name: profile.handle.to_owned(),
        actor_type: ActorType::Person,
        summary: Some("We can't rewind, we've gone too far".to_owned()),
        preferred_username: profile.handle,
        icon: None,
        inbox,
        outbox,
        public_key
    };

    let body = json!(actor).to_string();
    let mut res = send(body);
    let header_value = HeaderValue::from_static("application/activity+json");
    res.headers_mut().append(CONTENT_TYPE, header_value);
    Ok(res)
}
