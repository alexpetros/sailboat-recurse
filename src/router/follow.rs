use serde::{Deserialize, Serialize};

use crate::server::{request::IncomingRequest, response::{send, ResponseResult}};

#[derive(Serialize, Deserialize)]
struct Actor {
    url: String,
    name: String,
    handle: String,
    summary: String,
    host: String,
}

pub async fn post(req: IncomingRequest<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: Actor = serde_html_form::from_str(&text)?;

    req.db.execute(
        "INSERT INTO followed_actors (url, name, handle, summary, host)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        (&form.url, &form.name, &form.handle, &form.summary, &form.host)
    )?;

    let res = "<button disabled>Followed!</button>".to_string();
    Ok(send(res))
}
