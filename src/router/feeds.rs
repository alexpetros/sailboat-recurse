use minijinja::context;
use crate::activitypub::get_full_handle;
use crate::queries;
use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{send, ServerResponse};

pub async fn get(mut req: IncomingRequest<'_>) -> ServerResponse {
    let url_param = req.uri().path().split('/').last().unwrap();
    let handle = get_full_handle(url_param)?;

    let actor = queries::get_or_search_for_actor(&mut req.db, &req.domain, &handle).await?;
    let actor = match actor {
        None => return Ok(send("No account found")),
        Some(actor) => actor
    };

    let context = context!{ actor };
    let body = req.render("feeds/_feed_handle.html", context)?;
    Ok(send(body))
}
