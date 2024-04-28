use minijinja::context;

use crate::queries::get_posts_in_profile;
use crate::query_row;
use crate::server::server_request::{AuthedRequest, AuthStatus, PlainRequest, SetupStatus};
use crate::server::server_response::{self, redirect, ServerResult};

pub async fn get(req: PlainRequest<'_>) -> ServerResult {
    let req = match req.authenticate() {
        AuthStatus::Success(r) => r,
        AuthStatus::Failure(r) => return get_unauthed(r)
    };

    match req.has_passed_setup()? {
        SetupStatus::Complete(r) => get_authed(r).await,
        SetupStatus::Incomplete(_) => redirect("/profiles/new")
    }
}

pub fn get_unauthed(req: PlainRequest) -> ServerResult {
    let body = req.render("index/index.html", context! {})?;
    Ok(server_response::send(body))
}

pub async fn get_authed(req: AuthedRequest<'_>) -> ServerResult {
    let posts = get_posts_in_profile(&req.db, req.data.current_profile.profile_id)?;
    let mut query = req.db.prepare("SELECT count(*) FROM followed_actors")?;
    let follow_count: i64 = query.query_row((), |row| row.get(0))?;

    let profile = query_row!(
        req.db,
        Profile {
            profile_id: i64,
            preferred_username: String,
            display_name: String,
            nickname: String
        },
        "FROM profiles where profile_id = ?1",
        [req.data.current_profile.profile_id]
    );

    let profile = match profile {
        Ok(x) => x,
        Err(_) => return redirect("/profiles/new"),
    };

    let context = context! {
        posts,
        profile,
        name => "Alex",
        follow_count,
    };

    let body = req.render("index/index_auth.html", context)?;
    Ok(server_response::send(body))
}
