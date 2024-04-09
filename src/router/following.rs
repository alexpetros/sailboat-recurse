use minijinja::context;
use serde::Serialize;

use crate::server::request::IncomingRequest;
use crate::server::response::{ResponseResult, send};

#[derive(Serialize)]
struct Actor {
    url: String,
    name: String,
    handle: String,
}

pub async fn get(req: IncomingRequest<'_>) -> ResponseResult {
    let mut query = req.db.prepare("SELECT url, name, handle FROM followed_actors")?;
    let rows = query.query_map((), |row| {
        let actor = Actor {
            url: row.get(0)?,
            name: row.get(1)?,
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
