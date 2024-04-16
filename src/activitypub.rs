use crate::server::error::{bad_request, ServerError};

pub mod signature;
pub mod requests;
pub mod objects;

static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

#[derive(Debug)]
pub struct FullHandle {
    pub preferred_username: String,
    pub host: String
}

pub fn get_full_handle (username: &str) -> Result<FullHandle, ServerError> {
    let mut splits = username.splitn(3, '@').peekable();
    splits.next_if_eq(&"");
    let preferred_username = splits.next().ok_or(bad_request("Missing user name"))?.trim().to_owned();
    let host = splits.next().ok_or(bad_request("Missing host name"))?.trim().to_owned();

    Ok(FullHandle { preferred_username, host })
}

impl FullHandle {
    pub fn get_local_url (&self) -> String {
        format!("/feeds/@{}@{}", self.preferred_username, self.host)
    }
}

impl ToString for FullHandle {
    fn to_string(&self) -> String {
        format!("@{}@{}", self.preferred_username, self.host)
    }
}
