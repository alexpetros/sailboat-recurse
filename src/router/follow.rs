use serde::{Deserialize, Serialize};

use crate::server::{context::Context, request::IncomingRequest, response::{send, ResponseResult}};

#[derive(Serialize, Deserialize)]
struct Actor {
    url: String,
    display_name: String,
    handle: String,
    host: String
}

pub async fn post(req: IncomingRequest, ctx: Context<'_>) -> ResponseResult {
    let req = req.get_body().await?;
    let text = req.text()?;
    let form: Actor = serde_html_form::from_str(&text)?;

    ctx.db.execute(
        "INSERT INTO followed_actors (url, display_name, handle)
        VALUES (?1, ?2, ?3)",
        (&form.url, &form.display_name, &form.handle)
    )?;

    let res = "<button disabled>Followed!</button>".to_string();
    Ok(send(res))
}
