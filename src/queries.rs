use crate::activitypub::objects::actor::{Actor, LinkType};
use crate::activitypub::requests::{get_actor, get_webfinger};
use crate::activitypub::FullHandle;
use crate::query_row;
use crate::server::error::{bad_request, map_bad_gateway, ServerError};
use crate::server::server_request::CurrentProfile;
use crate::server::server_response::InternalResult;
use crate::templates::_partials::post::Post;
use hyper::Uri;
use rusqlite::Connection;
use tracing::warn;

pub fn get_posts_in_profile(db: &Connection, profile_id: i64, is_owner: bool) -> Result<Vec<Post>, ServerError> {
    let mut query = db.prepare(
        "SELECT post_id,
            display_name as actor_name,
            preferred_username as actor_handle,
            content,
            created_at
         FROM posts
         LEFT JOIN profiles AS f USING (profile_id)
         WHERE profile_id = ?1
         ORDER BY created_at DESC
         ",
    )?;

    let rows = query.query_map([profile_id], |row| {
        let post = Post {
            post_id: row.get(0)?,
            actor_name: row.get(1)?,
            actor_handle: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            is_owner,
        };
        Ok(post)
    })?;

    let posts: Vec<Post> = rows.collect::<Result<_, _>>()?;
    Ok(posts)
}

pub async fn get_or_search_for_actor(
    handle: &FullHandle,
    current_profile: &CurrentProfile,
) -> Result<Option<Actor>, ServerError> {
    // First check if we are following them
    let FullHandle {
        preferred_username,
        host,
    } = handle;
    // let actor = db.query_row("
    // SELECT name, url, preferred_username, host, summary
    // FROM followed_actors
    // WHERE handle = ? AND host = ?",
    // (preferred_username, host), |row| {
    //     let actor = Actor {
    //         name: row.get(0)?,
    //         url: row.get(1)?,
    //         preferred_username: row.get(2)?,
    //         host: row.get(3)?,
    //         summary: row.get(4)?,
    //         icon_url: "".to_owned(),
    //         is_followed: true
    //     };
    //     Ok(actor)
    // });

    // if actor.is_ok() { return Ok(actor.ok()); }

    let web_finger = get_webfinger(host, preferred_username).await?;
    let actor_link = web_finger
        .links
        .as_ref()
        .and_then(|links| {
            links
                .iter()
                .find(|l| l.rel == "self" && l.link_type == Some(LinkType::ActivityJson))
        })
        .and_then(|link| link.href.clone());

    let uri = match actor_link.as_ref() {
        None => return Ok(None),
        Some(link) => link.clone().parse::<Uri>(),
    }
    .map_err(|e| {
        warn!(
            "Invalid URI provided for self by {}@{}: {:?}",
            preferred_username, host, &actor_link
        );
        map_bad_gateway(e)
    })?;

    let actor = get_actor(&uri, current_profile).await?;

    // let actor = Actor {
    //     name: actor.name,
    //     url: local_url,
    //     preferred_username: actor.preferred_username,
    //     host: host.to_owned(),
    //     summary: actor.summary.unwrap_or("".to_owned()),
    //     icon_url,
    //     is_followed: false,
    //     outbox: actor.outbox.unwrap()
    // };

    Ok(Some(actor))
}

pub fn get_profile_id_from_url(db: &Connection, url: &str) -> InternalResult<i64> {
    // let preferred_username = _get_preferred_username_from_url(url)?;
    let uri: Uri = url.parse().map_err(|_| bad_request("Invalid URI provided"))?;
    let profile_id = uri.path().split('/').last();
    let profile = query_row!(
        db,
        Profile { profile_id: i64 },
        "FROM profiles WHERE profile_id = ?1",
        [profile_id]
        )?;

    Ok(profile.profile_id)
}

fn _get_preferred_username_from_url(url: &str) -> InternalResult<String> {
    let uri: Uri = url.parse().map_err(|_| bad_request("Invalid URI provided"))?;
    let path = &uri.path()[1..];
    let path_split = path.split_once('/').ok_or(bad_request("Non-profile URI provided"))?;

    let (route, preferred_username) = path_split;
    if route != "profiles" {
        return Err(bad_request("Non-profile URI provided"));
    }

    Ok(preferred_username.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_url() {
        let url = "not a url";
        let username = _get_preferred_username_from_url(url);
        assert_eq!(username, Err(bad_request("Invalid URI provided")))
    }

    #[test]
    fn schemaless_url() {
        let url = "example.com/not-profiles/1";
        let username = _get_preferred_username_from_url(url);
        assert_eq!(username, Err(bad_request("Invalid URI provided")))
    }

    #[test]
    fn non_profile_url() {
        let url = "http://example.com/not-profiles/alex";
        let username = _get_preferred_username_from_url(url);
        assert_eq!(username, Err(bad_request("Non-profile URI provided")))
    }

    #[test]
    fn profile_url() {
        let url = "http://example.com/profiles/alex";
        let username = _get_preferred_username_from_url(url);
        assert_eq!(username, Ok("alex".to_owned()))
    }
}
