use hyper::{StatusCode, Uri};
use serde::Deserialize;
use serde_json::json;
use tracing::warn;

use crate::{activitypub::{objects::{outbox::{AcceptActivity, ActivityType, FollowActivity, UndoActivity}, AtContext, Context}, requests::{self, send_as}}, router::debug, server::{error::{bad_gateway, bad_request}, server_request::{AnyRequest, AuthState, CurrentProfile, ServerRequest}, server_response::{send_status, ServerResult}}};
use crate::queries::get_profile_id_from_url;

#[derive(Deserialize)]
#[serde(untagged)]
enum InboxPost {
    Follow(FollowActivity),
    Undo(UndoActivity<FollowActivity>)
}

pub async fn post<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let req = req.into_text().await?;
    let body: InboxPost = req.parse_json()?;
    match body {
        InboxPost::Follow(activity) => follow(req, activity).await,
        InboxPost::Undo(activity) => undo_follow(req, activity)
    }
}

async fn follow<Au: AuthState>(req: ServerRequest<'_, String, Au>, follow_activity: FollowActivity) -> ServerResult {
    let actor_uri: Uri = follow_activity.actor.parse()
        .map_err(|_| bad_request("Invalid actor URI provided"))?;
    let profile_id = get_profile_id_from_url(&req.db, &follow_activity.object)?;

    let profile = CurrentProfile::new(&req.db, profile_id, &req.domain).ok_or_else(|| {
        bad_request(&format!("Feed {} not found", profile_id))
    })?;


    let actor = requests::get_actor(&actor_uri, &profile).await
        .map_err(|_| {
            let message = format!("Unable to retrieve parse valid actor from URI {}", actor_uri);
            bad_gateway(&message)
        })?;

    let inbox_uri = actor.inbox.parse::<Uri>()
        .map_err(|_| {
            let message = format!("{} is not a valid inbox URI", actor.inbox);
            bad_gateway(&message)
        })?;

    req.db.execute(
        "INSERT OR REPLACE INTO known_actors (actor_id, name, preferred_username, url, inbox, outbox)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (actor.id, &actor.name, &actor.preferred_username, &actor.url, &actor.inbox, &actor.outbox),
    )?;

    req.db.execute(
        "INSERT OR REPLACE INTO followers (profile_id, actor_id) VALUES (?1, ?2)",
        (profile_id, &follow_activity.actor))?;

    let accept = AcceptActivity {
        context: AtContext::Context(Context::ActivityStreams),
        activity_type: ActivityType::Accept,
        id: "1223234932098".to_owned(), // TODO generate these
        actor: format!("{}/profiles/{}", req.domain, profile_id),
        object: follow_activity
    };

    let accept_body = json!(accept).to_string();

    tokio::spawn(async move {
        let res = send_as(&inbox_uri, &profile, accept_body).await;
        let res = match res {
            Ok(r) => r,
            Err(e) => { return warn!("Failed to send confirmation to {}: {:?}", actor_uri, e) }
        };

        let code = res.status();
        let res_body = res.text().await;
        match res_body {
            Ok(body) => debug!("Received {} {}", code, body),
            Err(e) => warn!("Unable to reach {}: {:?}", actor_uri, e)
        };

    });

    send_status(StatusCode::OK)
}

fn undo_follow<Au: AuthState>(req: ServerRequest<'_, String, Au>, undo_activity: UndoActivity<FollowActivity>) -> ServerResult {
    let profile_id = get_profile_id_from_url(&req.db, &undo_activity.object.object)?;
    req.db.execute("DELETE FROM followers WHERE profile_id = ?1 AND actor_id = ?2",
                   (profile_id, &undo_activity.actor))?;

    send_status(StatusCode::OK)
}
