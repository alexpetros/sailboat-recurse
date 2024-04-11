use serde::{Deserialize, Serialize};

use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{ServerResponse, send};

#[derive(Serialize, Deserialize)]
struct Actor {
    url: String,
    name: String,
    handle: String,
    summary: String,
    host: String,
}

pub async fn post(req: IncomingRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
    let form: Actor = req.get_form_data()?;

    req.db.execute(
        "INSERT INTO followed_actors (url, name, handle, summary, host)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        (&form.url, &form.name, &form.handle, &form.summary, &form.host)
    )?;

    let res = "<button disabled>Followed!</button>".to_string();
    Ok(send(res))
}
