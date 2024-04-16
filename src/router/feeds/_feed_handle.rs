use hyper::Uri;
use minijinja::context;

use crate::activitypub::get_full_handle;
use crate::activitypub::objects::outbox::{OrderedCollectionPage, Outbox, PageOrLink};
use crate::activitypub::objects::outbox::Object::Note;
use crate::activitypub::requests::get_from_ap;
use crate::queries;
use crate::server::error::bad_gateway;
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

    let profile_id = req.locals.current_profile;
    let private_key_pem: String = req.db.query_row_and_then(
        "SELECT private_key_pem FROM profiles WHERE profile_id = ?1",
        [profile_id],
        |row| row.get(0))?;

    let outbox_uri = match &actor.outbox {
        Some(s) => s.clone().parse::<Uri>(),
        None => return Err(bad_gateway("No outbox provided"))
    }.map_err(|_| bad_gateway("Invalid outbox URI"))?;
    let outbox: Outbox = get_from_ap(&req.domain, profile_id, &outbox_uri, &private_key_pem).await?;

    let first_page_url = match outbox.first {
        PageOrLink::Link(s) => s.clone().parse::<Uri>(),
        PageOrLink::Page(_p) => todo!()
    }.map_err(|_| bad_gateway("Invalid outbox page URI"))?;

    let page: OrderedCollectionPage = get_from_ap(&req.domain, profile_id, &first_page_url, &private_key_pem).await?;
    let posts: Vec<_> = page.ordered_items.into_iter().filter_map(|a| {
        let note = match a.object {
            Note(n) => n,
            _ => return None
        };
        let context = context! {
            actor_name => actor.name,
            actor_handle => handle.to_string(),
            content => note.content,
            created_at => note.published,
            avi_url => actor.icon.as_ref().unwrap().url.clone()
        };
        Some(context)
    }).collect();
    let actor = context! { handle => handle.to_string(), name => actor.name };

    let context = context!{ actor, posts };
    let body = req.render("feeds/_feed_handle.html", context)?;
    Ok(send(body))
}
