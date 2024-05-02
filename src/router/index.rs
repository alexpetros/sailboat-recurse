use minijinja::context;
use rusqlite::named_params;

use crate::queries::get_posts_in_profile;
use crate::query_row_custom;
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
    let current_profile_id = req.data.current_profile.profile_id;
    let posts = get_posts_in_profile(&req.db, current_profile_id)?;

    let profile = query_row_custom!(
        req.db,
        Profile {
            profile_id: i64,
            preferred_username: String,
            display_name: String,
            nickname: String,
            following_count: i64,
            follower_count: i64
        },
        "
        SELECT
            profile_id,
            preferred_username,
            display_name,
            nickname,
            (SELECT count(*) FROM following WHERE profile_id = :id) as following_count,
            (SELECT count(*) FROM followers WHERE profile_id = :id) as follower_count
        FROM profiles
        where profile_id = :id",
        named_params!{ ":id": current_profile_id }
    );

    let profile = match profile {
        Ok(x) => x,
        Err(_) => return redirect("/profiles/new"),
    };

    let context = context! {
        posts,
        profile,
        name => "Alex",
    };

    let body = req.render("index/index_auth.html", context)?;
    Ok(server_response::send(body))
}
