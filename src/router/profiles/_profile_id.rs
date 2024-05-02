use crate::activitypub::objects::actor::{Actor, ActorType, PublicKey};
use crate::activitypub::objects::Context;
use crate::queries::get_posts_in_profile;
use crate::server::error::bad_request;
use crate::server::server_request::{AnyRequest, AuthState};
use crate::server::server_response::{self, not_found};
use crate::server::server_response::{send, ServerResult};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use minijinja::context;
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub mod inbox;
pub mod outbox;
pub mod following;
pub mod followers;

#[derive(Serialize, Deserialize)]
struct Profile {
    profile_id: i64,
    preferred_username: String,
    display_name: String,
    nickname: String,
    following_count: i64,
    follower_count: i64,
    private_key_pem: String,
}

pub async fn get<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let profile_param = req.get_trailing_param("Missing profile ID")?;

    let profile_id = match profile_param.split_once('#') {
        None => profile_param,
        Some((f, _)) => f,
    }
    .parse::<i64>()
    .map_err(|_| bad_request("Invalid profile ID"))?;

    let profile = req.db.query_row(
        "
        SELECT
            profile_id,
            preferred_username,
            display_name,
            nickname,
            (SELECT count(*) FROM following WHERE profile_id = :id) as following_count,
            (SELECT count(*) FROM followers WHERE profile_id = :id) as follower_count,
            private_key_pem
        FROM profiles
        where profile_id = :id",
        named_params!{ ":id": profile_id },
        |row| {
            let profile = Profile {
                profile_id: row.get(0)?,
                preferred_username: row.get(1)?,
                display_name: row.get(2)?,
                nickname: row.get(3)?,
                following_count: row.get(4)?,
                follower_count: row.get(5)?,
                private_key_pem: row.get(6)?,
            };
            Ok(profile)
        },
    );

    let profile = match profile {
        Ok(x) => x,
        Err(_) => return not_found(req)
    };

    match req.is_ap_req() {
        true => serve_json_profile(req, profile),
        false => serve_html_profile(req, profile).await
    }
}

async fn serve_html_profile<Au: AuthState>(req: AnyRequest<'_, Au>, profile: Profile) -> ServerResult {
    // let domain = req.domain;
    let posts = get_posts_in_profile(&req.db, profile.profile_id)?;

    let context = context! { profile => profile, posts => posts };

    let body = req.render("profiles/_profile_id.html", context)?;
    Ok(server_response::send(body))
}

fn serve_json_profile<Au: AuthState>(req: AnyRequest<'_, Au>, profile: Profile) -> ServerResult {
    let domain = req.domain;
    // let domain = query_row!(
    //     req.db,
    //     Domain { domain: String },
    //     "FROM profiles WHERE profile_id = ?1",
    //     (profile.profile_id, )
    //     )?.domain;

    let id = format!("https://{}/profiles/{}", domain, profile.profile_id);
    let inbox = format!("https://{}/inbox", domain);
    let outbox = format!("https://{}/profiles/{}/outbox", domain, profile.profile_id);
    let public_key = PublicKey::new(&id, &profile.private_key_pem);

    let context = vec![Context::ActivityStreams, Context::SecurityV1];
    let actor = Actor {
        context,
        id: id.to_owned(),
        url: id.to_owned(),
        name: profile.display_name,
        actor_type: ActorType::Person,
        summary: Some("We can't rewind, we've gone too far".to_owned()),
        preferred_username: profile.preferred_username,
        icon: None,
        inbox,
        outbox,
        public_key,
    };

    let body = json!(actor).to_string();
    let mut res = send(body);
    let header_value = HeaderValue::from_static("application/activity+json");
    res.headers_mut().append(CONTENT_TYPE, header_value);
    Ok(res)
}
