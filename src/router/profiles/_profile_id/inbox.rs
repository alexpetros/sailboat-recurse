use hyper::StatusCode;

use crate::{activitypub::objects::outbox::FollowActivity, server::{error::map_bad_request, server_request::{AnyRequest, AuthState}, server_response::{send_status, ServerResult}}};
use crate::queries::get_profile_id_from_url;

pub async fn post<Au: AuthState>(req: AnyRequest<'_, Au>) -> ServerResult {
    let req = req.into_text().await?;
    let follow_req: FollowActivity = serde_json::from_str(req.body()).map_err(map_bad_request)?;

    let profile_id = get_profile_id_from_url(&req.db, &follow_req.object)?;

    req.db.execute("INSERT OR REPLACE INTO followers (profile_id, following_actor) VALUES (?1, ?2)",
                   (profile_id, follow_req.actor))?;

    send_status(StatusCode::OK)
}
