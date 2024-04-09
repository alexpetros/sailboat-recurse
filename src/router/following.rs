use minijinja::context;
use serde::Serialize;

use crate::server::{request::IncomingRequest, response::{send, ResponseResult}};

#[derive(Serialize)]
struct Actor {
    url: String,
    display_name: String,
    handle: String,
}

pub async fn get(req: IncomingRequest<'_>) -> ResponseResult {
    let mut query = req.db.prepare("SELECT url, display_name, handle FROM followed_actors")?;
    let rows = query.query_map((), |row| {
        let actor = Actor {
            url: row.get(0)?,
            display_name: row.get(1)?,
            handle: row.get(2)?,
        };
        Ok(actor)
    })?;

    let mut following = Vec::new();
    for actor in rows {
        following.push(actor?);
    }

    let context = context! { following };
    let body = req.render("following.html", context);
    Ok(send(body))
}
