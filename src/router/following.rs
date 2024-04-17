use minijinja::context;
use serde::Serialize;

use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{send, ServerResponse};

#[derive(Serialize)]
struct Actor {
    url: String,
    name: String,
    handle: String,
}

pub async fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let mut query = req
        .db
        .prepare("SELECT url, name, handle FROM followed_actors")?;
    let rows = query.query_map((), |row| {
        let actor = Actor {
            url: row.get(0)?,
            name: row.get(1)?,
            handle: row.get(2)?,
        };
        Ok(actor)
    })?;

    // let following: Vec<Actor> = rows.collect();

    let mut following = Vec::new();
    for actor in rows {
        following.push(actor?);
    }

    let context = context! { following };
    let body = req.render("following.html", context)?;
    Ok(send(body))
}
