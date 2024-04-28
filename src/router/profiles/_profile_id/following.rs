use minijinja::context;
use crate::query_map;

use crate::server::server_request::AuthedRequest;
use crate::server::server_response::{send, ServerResult};

pub async fn get(req: AuthedRequest<'_>) -> ServerResult {
    let profile_id = req.get_url_param(2, "Invalid Profile ID")?;
    let following = query_map!(
        req.db,
        Actor { url: String, name: String, preferred_username: String },
        "FROM followed_actors WHERE profile_id = ?",
        [ profile_id ]
    );

    let context = context! { following };
    let body = req.render("profiles/_profile_id/following.html", context)?;
    Ok(send(body))
}
