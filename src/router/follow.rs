use serde::{Deserialize, Serialize};

use crate::server::server_request::AuthedRequest;
use crate::server::server_response::{send, ServerResult};

#[derive(Serialize, Deserialize)]
struct Actor {
    id: String,
    url: String,
    name: String,
    preferred_username: String,
    inbox: String,
    outbox: String,
    summary: String,
}

pub async fn post(req: AuthedRequest<'_>) -> ServerResult {
    let req = req.into_text().await?;
    let form: Actor = req.get_form_data()?;

    req.db.execute(
        "INSERT OR REPLACE INTO known_actors
            (actor_id, url, preferred_username, name, inbox, outbox, summary)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&form.id, &form.url, &form.preferred_username, &form.name, &form.inbox, &form.outbox, &form.summary),
    )?;

    req.db.execute(
        "INSERT INTO following (profile_id, actor_id)
        VALUES (?1, ?2)",
        (req.data.current_profile.profile_id, form.id),
    )?;


    let res = "<button disabled>Followed!</button>".to_string();
    Ok(send(res))
}
