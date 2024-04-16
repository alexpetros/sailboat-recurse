use minijinja::context;
use serde::Serialize;

use crate::queries::get_posts_in_profile;
use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{self, redirect, ServerResponse};

#[derive(Serialize)]
struct Profile {
    profile_id: i64,
    preferred_username: String,
    display_name: String,
    internal_name: String,
}

pub async fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let posts = get_posts_in_profile(&req.db, req.locals.current_profile)?;
    let mut query = req.db.prepare("SELECT count(*) FROM followed_actors")?;
    let follow_count: i64 = query.query_row((), |row| { row.get(0) })?;


    let profile = req.db.query_row("
        SELECT profile_id, preferred_username, display_name, internal_name
        FROM profiles where profile_id = ?1"
        , [ req.locals.current_profile ], |row| {
            let profile = Profile {
                profile_id: row.get(0)?,
                preferred_username: row.get(1)?,
                display_name: row.get(2)?,
                internal_name: row.get(3)?,
            };
            Ok(profile)
        });

    let profile = match profile {
        Ok(x) => x,
        Err(_) => return redirect("/profiles/new")
    };

    let context = context! {
        posts,
        profile,
        name => "Alex",
        follow_count,
    };

    let body = req.render("index.html", context)?;
    Ok(server_response::send(body))
}
