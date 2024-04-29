use crate::server::error::{bad_request, ServerError};
use std::fmt::Display;

pub mod objects;
pub mod requests;
pub mod signature;

static SHORT_ACCEPT_HEADER: &str = "application/activity+json";
static PUBLIC_STREAM: &str = "https://www.w3.org/ns/activitystreams#Public";

#[derive(Debug)]
pub struct FullHandle {
    pub preferred_username: String,
    pub host: String,
}

pub fn get_full_handle(username: &str) -> Result<FullHandle, ServerError> {
    let mut splits = username.splitn(3, '@').peekable();
    splits.next_if_eq(&"");
    let preferred_username = splits
        .next()
        .ok_or(bad_request("Missing user name"))?
        .trim()
        .to_owned();
    let host = splits
        .next()
        .ok_or(bad_request("Missing host name"))?
        .trim()
        .to_owned();

    Ok(FullHandle {
        preferred_username,
        host,
    })
}

impl FullHandle {
    pub fn get_local_url(&self) -> String {
        format!("/feeds/@{}@{}", self.preferred_username, self.host)
    }
}

impl Display for FullHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("@{}@{}", self.preferred_username, self.host)
        )
    }
}
