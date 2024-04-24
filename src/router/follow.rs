use serde::{Deserialize, Serialize};

use crate::server::server_request::AuthedRequest;
use crate::server::server_response::{send, ServerResult};

#[derive(Serialize, Deserialize)]
struct Actor {
    url: String,
    name: String,
    preferred_username: String,
    summary: String,
    host: String,
}

pub async fn post(req: AuthedRequest<'_>) -> ServerResult {
    let req = req.to_text().await?;
    let form: Actor = req.get_form_data()?;

    req.db.execute(
        "INSERT INTO followed_actors (url, name, preferred_username, summary, host)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &form.url,
            &form.name,
            &form.preferred_username,
            &form.summary,
            &form.host,
        ),
    )?;

    let res = "<button disabled>Followed!</button>".to_string();
    Ok(send(res))
}
