use crate::server::context::GlobalContext;
use crate::server::error;
use crate::server::error::{map_bad_gateway, map_bad_request, ServerError};
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::header::COOKIE;
use hyper::{Method, Uri};
use minijinja::{context, Value};
use openssl::pkey::{PKey, Private};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tracing::log::warn;

use super::error::{body_not_utf8, body_too_large};

const ENV: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "prod"
};
// TODO this wouldn't work if you deleted for your first profile
const DEFAULT_PROFILE: i64 = 1;

pub enum SR<'a, T> {
    Authed(AuthenticatedRequest<'a, T>),
    Unauthed(UnauthenticatedRequest<'a, T>)
}

pub type AuthedRequest<'a> = AuthenticatedRequest<'a, Incoming>;
pub type UnauthedRequest<'a> = UnauthenticatedRequest<'a, Incoming>;

// TODO https://users.rust-lang.org/t/how-to-dispatch-enum-function-to-all-variants/80152/3
impl<'a, T> SR<'a, T>{
    pub fn uri(&self) -> &Uri {
        match self {
            SR::Authed(x) => x.uri(),
            SR::Unauthed(x) => x.uri()
        }
    }
    pub fn method(&self) -> &Method {
        match self {
            SR::Authed(x) => x.method(),
            SR::Unauthed(x) => x.method()
        }
    }
    pub fn statics(&self) -> &Arc<HashMap<std::string::String, Vec<u8>>> {
        match self {
            SR::Authed(x) => &x.global.statics,
            SR::Unauthed(x) => &x.global.statics,
        }
    }
    pub fn render(&self, path: &str, local_values: Value) -> Result<Vec<u8>, ServerError> {
        match self {
            SR::Authed(x) => x.render(path, local_values),
            SR::Unauthed(x) => x.render(path, local_values)
        }
    }
}

pub fn get_current_profile(db: &Connection, cookies: &HashMap<String, String>, domain: String) -> Option<CurrentProfile> {

    let current_profile_id = cookies
        .get("current_profile")
        .map(|id| id.parse::<i64>().ok())
        .flatten()?;

    let private_key_pem: String = db.query_row(
        "SELECT private_key_pem FROM profiles WHERE profile_id = ?1",
        (current_profile_id,),
        |row| Ok(row.get(0)?),
        ).ok()?;
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes()).ok()?;
    let current_profile = CurrentProfile {
        profile_id: current_profile_id,
        pkey,
        domain,
    };
    Some(current_profile)
}

pub fn get_request<'a, T>(
        request: hyper::Request<T>,
        g_ctx: &Arc<GlobalContext<'a>>,
        db: Connection,
        domain: String,
        ) -> Result<SR<'a, T>, ServerError> {
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
    let current_profile = get_current_profile(&db, &cookies, domain.clone());
    if let Some(profile) = current_profile {
        let req = AuthenticatedRequest::new(request, g_ctx, db, domain, cookies, profile)?;
        Ok(SR::Authed(req))
    } else {
        let req = UnauthenticatedRequest::new(request, g_ctx, db, domain)?;
        Ok(SR::Unauthed(req))
    }
}


pub struct UnauthenticatedRequest<'a, T> {
    pub request: hyper::Request<T>,
    pub global: Arc<GlobalContext<'a>>,
    pub db: Connection,
}

impl <'a, T> UnauthenticatedRequest<'a, T> {
    pub fn new(
        request: hyper::Request<T>,
        g_ctx: &Arc<GlobalContext<'a>>,
        db: Connection,
        domain: String,
    ) -> Result<Self, ServerError> {
        Ok(Self { request, global: g_ctx.clone(), db, })
    }
    pub fn get_trailing_param(&self, message: &str) -> Result<&str, ServerError> {
        self.uri()
            .path()
            .split("/")
            .last()
            .ok_or(error::bad_request(message))
    }

    pub fn render(&self, path: &str, local_values: Value) -> Result<Vec<u8>, ServerError> {
        let tmpl = self.global.env.get_template(path).unwrap();
        tmpl.render(())
            .map(|x| x.into_bytes())
            .map_err(|e| map_bad_gateway(e))
    }
}

impl<'a, T> Deref for UnauthenticatedRequest<'a, T> {
    type Target = hyper::Request<T>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl<'a> UnauthenticatedRequest<'a, Incoming> {
    pub async fn get_body(self) -> Result<UnauthenticatedRequest<'a, Bytes>, ServerError> {
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

        Ok(UnauthenticatedRequest { request, global, db, })
    }

    pub async fn to_text(self) -> Result<UnauthenticatedRequest<'a, String>, ServerError> {
        self.get_body().await?.to_text()
    }
}

// Not sure that I even need this intermediate state at all right now
// But I think it will become relevant for uploading images
impl<'a> UnauthenticatedRequest<'a, Bytes> {
    pub fn to_text(self) -> Result<UnauthenticatedRequest<'a, String>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let str = String::from_utf8(body.to_vec()).map_err(|_| body_not_utf8())?;
        let request = hyper::Request::from_parts(parts, str);

        let global = self.global;
        let db = self.db;
        Ok(UnauthenticatedRequest { request, global, db, })
    }
}

impl<'a> UnauthenticatedRequest<'a, String> {
    pub fn get_form_data<T>(&'a self) -> Result<T, ServerError>
    where
        T: Deserialize<'a>,
    {
        let str = self.body();
        serde_html_form::from_str::<T>(str).map_err(|e| {
            warn!("failed to deserialize request {}", &self.body());
            map_bad_request(e)
        })
    }
}

#[derive(Serialize)]
struct Profile {
    profile_id: i64,
    nickname: String,
}

pub struct CurrentProfile {
    pub profile_id: i64,
    pub domain: String,
    pub pkey: PKey<Private>,
}

#[derive(Serialize)]
struct Locals {
    profiles: Vec<Profile>,
}

pub struct AuthenticatedRequest<'a, T> {
    pub request: hyper::Request<T>,
    pub global: Arc<GlobalContext<'a>>,
    pub current_profile: CurrentProfile,
    pub db: Connection,
    pub cookies: HashMap<String, String>,
    locals: Locals,
}

impl<'a, T> AuthenticatedRequest<'a, T> {
    fn make_context(&self, local_values: Value) -> Value {
        let global_values = context! { env => ENV };
        let request_values = context! { profiles => self.locals.profiles };
        context! { ..local_values, ..request_values, ..global_values }
    }

    pub fn get_trailing_param(&self, message: &str) -> Result<&str, ServerError> {
        self.uri()
            .path()
            .split("/")
            .last()
            .ok_or(error::bad_request(message))
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

impl<'a, T> Deref for AuthenticatedRequest<'a, T> {
    type Target = hyper::Request<T>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl<'a, T> AuthenticatedRequest<'a, T> {
    pub fn new(
        request: hyper::Request<T>,
        g_ctx: &Arc<GlobalContext<'a>>,
        db: Connection,
        domain: String,
        cookies: HashMap<String, String>,
        current_profile: CurrentProfile,
    ) -> Result<Self, ServerError> {
        let profiles = {
            let mut query = db.prepare("SELECT profile_id, nickname FROM profiles")?;
            let rows = query.query_map((), |row| {
                let profiles = Profile {
                    profile_id: row.get(0)?,
                    nickname: row.get(1)?,
                };
                Ok(profiles)
            })?;

            let mut profiles = Vec::new();
            for profile in rows {
                profiles.push(profile?);
            }

            profiles
        };

        let locals = Locals { profiles };

        Ok(Self {
            request,
            global: g_ctx.clone(),
            db,
            locals,
            cookies,
            current_profile,
        })
    }
}

impl<'a> AuthenticatedRequest<'a, Incoming> {
    pub async fn get_body(self) -> Result<AuthenticatedRequest<'a, Bytes>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let body_bytes = http_body_util::Limited::new(body, 1024 * 64);

        let bytes = body_bytes
            .collect()
            .await
            .map(|r| r.to_bytes())
            .map_err(|_| body_too_large())?;

        let request = hyper::Request::from_parts(parts, bytes);
        let current_profile = self.current_profile;
        let global = self.global;
        let db = self.db;
        let locals = self.locals;
        let cookies = self.cookies;

        Ok(AuthenticatedRequest {
            request,
            global,
            db,
            current_profile,
            cookies,
            locals,
        })
    }

    pub async fn to_text(self) -> Result<AuthenticatedRequest<'a, String>, ServerError> {
        self.get_body().await?.to_text()
    }
}

// Not sure that I even need this intermediate state at all right now
// But I think it will become relevant for uploading images
impl<'a> AuthenticatedRequest<'a, Bytes> {
    pub fn to_text(self) -> Result<AuthenticatedRequest<'a, String>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let str = String::from_utf8(body.to_vec()).map_err(|_| body_not_utf8())?;
        let request = hyper::Request::from_parts(parts, str);

        let current_profile = self.current_profile;
        let global = self.global;
        let db = self.db;
        let locals = self.locals;
        let cookies = self.cookies;
        Ok(AuthenticatedRequest {
            request,
            global,
            db,
            current_profile,
            cookies,
            locals,
        })
    }
}

impl<'a> AuthenticatedRequest<'a, String> {
    pub fn get_form_data<T>(&'a self) -> Result<T, ServerError>
    where
        T: Deserialize<'a>,
    {
        let str = self.body();
        serde_html_form::from_str::<T>(str).map_err(|e| {
            warn!("failed to deserialize request {}", &self.body());
            map_bad_request(e)
        })
    }
}
