use minijinja::context;

use crate::queries::get_posts_in_profile;
use crate::server::server_request::IncomingRequest;
use crate::server::server_response;
use crate::server::server_response::ServerResponse;

pub async fn get(req: IncomingRequest<'_>) -> ServerResponse {
    let posts = get_posts_in_profile(&req.db, 1)?;
    let mut query = req.db.prepare("SELECT count(*) FROM followed_actors")?;
    let follow_count: i64 = query.query_row((), |row| { row.get(0) })?;

    let context = context! {
        posts,
        profile_id => "1",
        name => "Alex",
        follow_count,
    };

    let body = req.render("index.html", context);
    Ok(server_response::send(body))
}
