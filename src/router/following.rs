use minijinja::context;
use crate::query_map;

use crate::server::server_request::AuthedRequest;
use crate::server::server_response::{send, ServerResponse};

pub async fn get(req: AuthedRequest<'_>) -> ServerResponse {
    let following = query_map!(
        req.db,
        Actor { url: String, name: String, preferred_username: String },
        "FROM followed_actors",
        ()
    );

    let context = context! { following };
    let body = req.render("following.html", context)?;
    Ok(send(body))
}
