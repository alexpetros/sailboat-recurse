use crate::server::context::GlobalContext;
use crate::server::error;
use crate::server::error::{map_bad_gateway, map_bad_request, ServerError};
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::header::{ACCEPT, COOKIE};
use minijinja::{context, Value};
use openssl::pkey::{PKey, Private};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tracing::log::warn;

pub static LONG_ACCEPT_HEADER: &str = "application/ld+json;profile=“https://www.w3.org/ns/activitystreams";
pub static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

use super::error::{bad_request, body_not_utf8, body_too_large};
use super::server_response::InternalResult;

const ENV: &str = if cfg!(debug_assertions) { "debug" } else { "prod" };

#[derive(Serialize)]
pub struct Profile {
    pub profile_id: i64,
    pub nickname: String,
}

#[derive(Debug, Serialize)]
pub struct CurrentProfile {
    pub profile_id: i64,
    pub domain: String,
    #[serde(skip)] pub pkey: PKey<Private>,
}

impl CurrentProfile {
    pub fn new(db: &Connection, profile_id: i64, domain: &str) -> Option<Self> {
        let private_key_pem: String = db.query_row(
            "SELECT private_key_pem FROM profiles WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0)
            ).ok()?;
        let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes()).ok()?;
        let current_profile = CurrentProfile {
            profile_id,
            pkey,
            domain: domain.to_owned(),
        };

        Some(current_profile)
    }
}

pub struct NoAuth;
pub struct SetupPhase;

#[derive(Serialize)]
pub struct SessionData {
    pub profiles: Vec<Profile>,
    pub current_profile: CurrentProfile,
}

pub trait AuthState {
    fn get(&self) -> Option<&SessionData>;
}

impl AuthState for SetupPhase {
    fn get(&self) -> Option<&SessionData> { None }
}

impl AuthState for NoAuth {
    fn get(&self) -> Option<&SessionData> { None }
}

impl AuthState for SessionData {
    fn get(&self) -> Option<&SessionData> { Some(self) }
}

pub type AuthedRequest<'a> = ServerRequest<'a, Incoming, SessionData>;
pub type SetupRequest<'a> = ServerRequest<'a, Incoming, SetupPhase>;
pub type PlainRequest<'a> = ServerRequest<'a, Incoming, NoAuth>;
pub type AnyRequest<'a, Au> = ServerRequest<'a, Incoming, Au>;

pub struct ServerRequest<'a, T, Au: AuthState> {
    pub request: hyper::Request<T>,
    pub global: Arc<GlobalContext<'a>>,
    pub db: Connection,
    pub cookies: HashMap<String, String>,
    pub data: Au,
    pub domain: String,
}

pub fn new_request<T>(
    request: hyper::Request<T>,
    global: Arc<GlobalContext>,
    db: Connection,
    domain: String
) -> Result<ServerRequest<T, NoAuth>, ServerError> {
    let cookie_string = request
        .headers()
        .get(COOKIE)
        .and_then(|value| value.to_str().ok());

    let cookies = cookie_string
        .map(|s| s.split("; ").collect::<Vec<&str>>())
        .unwrap_or_default()
        .iter()
        .filter_map(|s| s.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect::<HashMap<String, String>>();

    Ok(ServerRequest { request, global, db, domain, cookies, data: NoAuth })
}

impl<'a, T, Au: AuthState> ServerRequest<'a, T, Au> {
    pub fn get_url_param(&self, pos: usize, message: &str) -> Result<&str, ServerError> {
        self.uri()
            .path()
            .split('/')
            .nth(pos)
            .ok_or(error::bad_request(message))
    }

    pub fn get_int_url_param(&self, pos: usize, message: &str) -> Result<i64, ServerError> {
        let str_param = self.get_url_param(pos, message)?;
        str_param.parse().map_err(|_| bad_request("Invalid URL parameter provided"))
    }

    pub fn get_trailing_param(&self, message: &str) -> Result<&str, ServerError> {
        self.uri()
            .path()
            .split('/')
            .last()
            .ok_or(error::bad_request(message))
    }

    fn make_context(&self, local_values: Value) -> Value {
        let global_values = context! { env => ENV };
        if let Some(locals) = self.data.get() {
            let request_values = context! { profiles => locals.profiles };
            context! { ..local_values, ..request_values, ..global_values }
        } else {
            context! { ..local_values, ..global_values }
        }
    }

    pub fn render(&self, path: &str, local_values: Value) -> Result<Vec<u8>, ServerError> {
        let tmpl = self.global.env.get_template(path).unwrap();
        let context = self.make_context(local_values);
        tmpl.render(context)
            .map(|x| x.into_bytes())
            .map_err(map_bad_gateway)
    }

    // Check whether the header is asking for (AcvitiyPub) JSON
    // TODO actually parse the header properly
    pub fn is_ap_req(&self) -> bool {
        let request_header = self.headers().get(ACCEPT);
        match request_header {
            None => false,
            Some(h) => {
                let h = h.to_str().unwrap_or("");
                h.contains(LONG_ACCEPT_HEADER) || h.contains(SHORT_ACCEPT_HEADER)
            }
        }
    }

// pub fn render_block(&self, path: &str, block_name: &str, local_values: Value) -> Vec<u8> {
//     let tmpl = self.global.env.get_template(path).unwrap();
//     let context = self.make_context(local_values);
//     tmpl.eval_to_state(context).unwrap().render_block(block_name).unwrap().into_bytes()
// }
}

impl<'a, T, Au: AuthState> Deref for ServerRequest<'a, T, Au> {
    type Target = hyper::Request<T>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

pub enum AuthStatus<'a, T> {
    Success(ServerRequest<'a, T, SetupPhase>),
    Failure(ServerRequest<'a, T, NoAuth>),
}

impl<'a, T> ServerRequest<'a, T, NoAuth> {
    pub fn authenticate(self) -> AuthStatus<'a, T> {
        let cookie_token = match self.cookies.get("token") {
            Some(token) => token,
            None => return AuthStatus::Failure(self)
        };

        let token_exists = self.db
            .query_row("SELECT token FROM sessions WHERE token = ?1", (cookie_token, ), |_| { Ok(true) })
            .optional()
            .ok()
            .flatten()
            .is_some();

        if !token_exists {
            return AuthStatus::Failure(self)
        }

        let request = self.request;
        let global = self.global;
        let db = self.db;
        let domain = self.domain;
        let cookies = self.cookies;
        let data = SetupPhase;

        AuthStatus::Success(ServerRequest { request, global, db, domain, cookies, data })
    }
}

pub enum SetupStatus<'a, T> {
    Complete(ServerRequest<'a, T, SessionData>),
    Incomplete(ServerRequest<'a, T, SetupPhase>),
}

impl<'a, T> ServerRequest<'a, T, SetupPhase> {
    pub fn has_passed_setup(self) -> Result<SetupStatus<'a, T>, ServerError> {
        let profiles = {
            let mut query = self.db.prepare("SELECT profile_id, nickname FROM profiles")?;
            let rows = query.query_map((), |row| {
                let profiles = Profile { profile_id: row.get(0)?, nickname: row.get(1)? };
                Ok(profiles)
            })?;

            let mut profiles = Vec::new();
            for profile in rows { profiles.push(profile?); }

            profiles
        };

        let request = self.request;
        let global = self.global;
        let db = self.db;
        let domain = self.domain;
        let cookies = self.cookies;

        let current_profile_id = cookies
            .get("current_profile")
            .and_then(|id| id.parse::<i64>().ok())
            .or_else(|| {
                db.query_row("SELECT profile_id FROM profiles", (), |row| { row.get(0) }).ok()
            });

        let current_profile = current_profile_id.map(|profile_id| {
            CurrentProfile::new(&db, profile_id, &domain)
        }).flatten();

        let current_profile = match current_profile {
            Some(p) => p,
            None => {
                let req = ServerRequest { request, global, db, domain, cookies, data: SetupPhase };
                return Ok(SetupStatus::Incomplete(req))
            }
        };

        let data = SessionData { profiles, current_profile };

        let req = ServerRequest { request, global, db, domain, cookies, data };
        Ok(SetupStatus::Complete(req))
    }
}

impl<'a, Au: AuthState> ServerRequest<'a, Incoming, Au> {
    pub async fn get_body(self) -> Result<ServerRequest<'a, Bytes, Au>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let body_bytes = http_body_util::Limited::new(body, 1024 * 64);

        let bytes = body_bytes
            .collect()
            .await
            .map(|r| r.to_bytes())
            .map_err(|_| body_too_large())?;

        let request = hyper::Request::from_parts(parts, bytes);
        let global = self.global;
        let db = self.db;
        let domain = self.domain;
        let cookies = self.cookies;
        let data = self.data;

        Ok(ServerRequest { request, global, db, domain, cookies, data })
    }

    pub async fn into_text(self) -> Result<ServerRequest<'a, String, Au>, ServerError> {
        self.get_body().await?.into_text()
    }
}

// Not sure that I even need this intermediate state at all right now
// But I think it will become relevant for uploading images
impl<'a, Au: AuthState> ServerRequest<'a, Bytes, Au> {
    pub fn into_text(self) -> Result<ServerRequest<'a, String, Au>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let str = String::from_utf8(body.to_vec()).map_err(|_| body_not_utf8())?;
        let request = hyper::Request::from_parts(parts, str);

        let global = self.global;
        let db = self.db;
        let domain = self.domain;
        let cookies = self.cookies;
        let data = self.data;

        Ok(ServerRequest { request, global, db, domain, cookies, data })
    }
}

impl<'a, Au: AuthState> ServerRequest<'a, String, Au> {
    pub fn get_form_data<T: Deserialize<'a>>(&'a self) -> InternalResult<T> {
        let str = self.body();
        serde_html_form::from_str::<T>(str).map_err(|e| {
            warn!("failed to deserialize request {}", &self.body());
            map_bad_request(e)
        })
    }

    pub fn parse_json<T: Deserialize<'a>>(&'a self) -> InternalResult<T> {
        let str = self.body();
        serde_json::from_str::<T>(str).map_err(|e| {
            warn!("failed to deserialize request body {}", &self.body());
            map_bad_request(e)
        })
    }
}
