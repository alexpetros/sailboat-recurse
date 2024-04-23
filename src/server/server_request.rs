use crate::server::context::GlobalContext;
use crate::server::error;
use crate::server::error::{map_bad_gateway, map_bad_request, ServerError};
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::header::COOKIE;
use minijinja::{context, Value};
use openssl::pkey::{PKey, Private};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tracing::log::warn;

use super::error::{body_not_utf8, body_too_large};

const ENV: &str = if cfg!(debug_assertions) { "debug" } else { "prod" };
// TODO this wouldn't work if you deleted for your first profile
// const DEFAULT_PROFILE: i64 = 1;

#[derive(Serialize)]
struct Profile {
    profile_id: i64,
    nickname: String,
}

#[derive(Serialize)]
pub struct CurrentProfile {
    pub profile_id: i64,
    pub domain: String,
    #[serde(skip)] pub pkey: PKey<Private>,
}

#[derive(Serialize)]
pub struct Locals {
    profiles: Vec<Profile>,
    pub current_profile: CurrentProfile,
}

pub enum AuthState {
    Authed(Locals),
    Setup,
    Plain,
}

pub type AuthedRequest<'a> = ServerRequest<'a, Incoming>;
pub type SetupRequest<'a> = ServerRequest<'a, Incoming>;
pub type PlainRequest<'a> = ServerRequest<'a, Incoming>;
pub type AnyRequest<'a> = ServerRequest<'a, Incoming>;

pub struct ServerRequest<'a, T> {
    pub request: hyper::Request<T>,
    pub global: Arc<GlobalContext<'a>>,
    pub db: Connection,
    pub cookies: HashMap<String, String>,
    pub auth_state: AuthState,
    pub domain: String,
}

impl<'a, T> ServerRequest<'a, T> {
    pub fn new(request: hyper::Request<T>, global: Arc<GlobalContext<'a>>, db: Connection, domain: String)
           -> Result<Self, ServerError> {
        let cookie_string = request
            .headers()
            .get(COOKIE)
            .map(|value| value.to_str().ok())
            .flatten();

        let cookies = cookie_string
            .map(|s| s.split("; ").collect::<Vec<&str>>())
            .unwrap_or(Vec::<&str>::new())
            .iter()
            .filter_map(|s| s.split_once("="))
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect::<HashMap<String, String>>();

        let mut req = ServerRequest { request, global, db, domain, cookies, auth_state: AuthState::Plain };

        let cookie_token = match req.cookies.get("token") {
            Some(token) => token,
            None => return Ok(req)
        };

        let token_exists = req.db
            .query_row("SELECT token FROM sessions WHERE token = ?1",
                       (cookie_token, ),
                       |_| { Ok(true) })
            .optional()?
            .is_some();

        if !token_exists {
            return Ok(req);
        }

        let profile = get_current_profile(&req.db, &req.cookies, &req.domain);
        let current_profile = match profile {
            Some(x) => x,
            None => {
                req.auth_state = AuthState::Setup;
                return Ok(req);
            }
        };

        let profiles = {
            let mut query = req.db.prepare("SELECT profile_id, nickname FROM profiles")?;
            let rows = query.query_map((), |row| {
                let profiles = Profile { profile_id: row.get(0)?, nickname: row.get(1)? };
                Ok(profiles)
            })?;

            let mut profiles = Vec::new();
            for profile in rows { profiles.push(profile?); }

            profiles
        };

        let locals = Locals { profiles, current_profile };
        req.auth_state = AuthState::Authed(locals);
        Ok(req)
    }

    pub fn current_profile(&self) -> Option<&CurrentProfile> {
        match &self.auth_state {
            AuthState::Authed(locals) => Some(&locals.current_profile),
            _ => None
        }
    }

    pub fn get_locals(&self) -> Option<&Locals> {
        match &self.auth_state {
            AuthState::Authed(locals) => Some(locals),
            _ => None
        }
    }

    pub fn get_trailing_param(&self, message: &str) -> Result<&str, ServerError> {
        self.uri()
            .path()
            .split("/")
            .last()
            .ok_or(error::bad_request(message))
    }

    fn make_context(&self, local_values: Value) -> Value {
        let global_values = context! { env => ENV };
        if let Some(locals) = self.get_locals() {
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
            .map_err(|e| map_bad_gateway(e))
    }

// pub fn render_block(&self, path: &str, block_name: &str, local_values: Value) -> Vec<u8> {
//     let tmpl = self.global.env.get_template(path).unwrap();
//     let context = self.make_context(local_values);
//     tmpl.eval_to_state(context).unwrap().render_block(block_name).unwrap().into_bytes()
// }

}

impl<'a, T> Deref for ServerRequest<'a, T> {
    type Target = hyper::Request<T>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl<'a> ServerRequest<'a, Incoming> {
    pub async fn get_body(self) -> Result<ServerRequest<'a, Bytes>, ServerError> {
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
        let auth_state = self.auth_state;

        Ok(ServerRequest { request, global, db, domain, cookies, auth_state })
    }

    pub async fn to_text(self) -> Result<ServerRequest<'a, String>, ServerError> {
        self.get_body().await?.to_text()
    }
}

// Not sure that I even need this intermediate state at all right now
// But I think it will become relevant for uploading images
impl<'a> ServerRequest<'a, Bytes> {
    pub fn to_text(self) -> Result<ServerRequest<'a, String>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let str = String::from_utf8(body.to_vec()).map_err(|_| body_not_utf8())?;
        let request = hyper::Request::from_parts(parts, str);

        let global = self.global;
        let db = self.db;
        let domain = self.domain;
        let cookies = self.cookies;
        let auth_state = self.auth_state;

        Ok(ServerRequest { request, global, db, domain, cookies, auth_state })
    }
}

impl<'a> ServerRequest<'a, String> {
    pub fn get_form_data<T: Deserialize<'a>>(&'a self) -> Result<T, ServerError> {
        let str = self.body();
        serde_html_form::from_str::<T>(str).map_err(|e| {
            warn!("failed to deserialize request {}", &self.body());
            map_bad_request(e)
        })
    }
}

pub fn get_current_profile(db: &Connection, cookies: &HashMap<String, String>, domain: &str) -> Option<CurrentProfile> {
    let current_profile_id = cookies
        .get("current_profile")
        .map(|id| id.parse::<i64>().ok())
        .flatten()?;

    let private_key_pem: String = db.query_row(
        "SELECT private_key_pem FROM profiles WHERE profile_id = ?1",
        (current_profile_id, ),
        |row| Ok(row.get(0)?),
    ).ok()?;
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes()).ok()?;
    let current_profile = CurrentProfile {
        profile_id: current_profile_id,
        pkey,
        domain: domain.to_owned(),
    };
    Some(current_profile)
}
