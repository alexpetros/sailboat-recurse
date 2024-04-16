use hyper::Uri;
use minijinja::context;

use crate::activitypub::get_full_handle;
use crate::activitypub::objects::outbox::Object::Note;
use crate::activitypub::objects::outbox::PageOrLink;
use crate::activitypub::requests::{get_outbox, get_outbox_page};
use crate::queries;
use crate::server::error::bad_gateway;
use crate::server::server_request::IncomingRequest;
use crate::server::server_response::{send, ServerResponse};

pub async fn get(mut req: IncomingRequest<'_>) -> ServerResponse {
    let url_param = req.uri().path().split('/').last().unwrap();
    let handle = get_full_handle(url_param)?;

    let actor = queries::get_or_search_for_actor(&mut req.db, &handle, &req.current_profile).await?;
    let actor = match actor {
        None => return Ok(send("No account found")),
        Some(actor) => actor
    };

    let outbox_uri = match &actor.outbox {
        Some(s) => s.clone().parse::<Uri>(),
        None => return Err(bad_gateway("No outbox provided"))
    }.map_err(|_| bad_gateway("Invalid outbox URI"))?;
    let outbox = get_outbox(&outbox_uri, &req.current_profile).await?;

    let first_page_url = match outbox.first {
        PageOrLink::Link(s) => s.clone().parse::<Uri>(),
        PageOrLink::Page(_p) => todo!()
    }.map_err(|_| bad_gateway("Invalid outbox page URI"))?;

    let page = get_outbox_page(&first_page_url, &req.current_profile).await?;
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
