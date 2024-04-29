use crate::activitypub::get_full_handle;
use minijinja::context;
use serde::Deserialize;

use crate::queries;
use crate::server::server_request::AuthedRequest;
use crate::server::server_response::send;
use crate::server::server_response::ServerResult;

#[derive(Deserialize)]
struct Query {
    q: String,
}

pub async fn get(req: AuthedRequest<'_>) -> ServerResult {
    let body = req.render("search.html", context! {})?;
    Ok(send(body))
}

pub async fn post(req: AuthedRequest<'_>) -> ServerResult {
    let req = req.to_text().await?;
    let query: Query = req.get_form_data()?;
    let handle = get_full_handle(&query.q)?;

    let actor = queries::get_or_search_for_actor(&handle, &req.data.current_profile).await?;
    let actor = match actor {
        None => return Ok(send("No account found")),
        Some(actor) => actor,
    };

    let icon_url = actor
        .icon
        .as_ref()
        .map(|i| i.url.clone())
        .unwrap_or("".to_owned());
    let local_url = handle.get_local_url();

    let actor = context! { local_url, icon_url, ..actor };
    let context = context! { actor };

    let body = req.render("_partials/feed-search-result.html", context)?;
    Ok(send(body))
}
