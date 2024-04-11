use std::ops::Deref;
use std::sync::Arc;
use hyper::body::{Bytes, Incoming};
use http_body_util::BodyExt;
use minijinja::{context, Value};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::log::warn;
use crate::server::context::GlobalContext;
use crate::server::error::{map_bad_request, ServerError};

use super::error::{body_not_utf8, body_too_large};

#[derive(Serialize)]
struct Profile {
    profile_id: i64,
    internal_name: String,
}

#[derive(Serialize)]
struct Locals {
    profiles: Vec<Profile>,
}

const ENV: &str = if cfg!(debug_assertions) { "debug" } else { "prod" };

pub type IncomingRequest<'a> = ServerRequest<'a, Incoming>;

pub struct ServerRequest<'a, T> {
    pub request: hyper::Request<T>,
    pub global: Arc<GlobalContext<'a>>,
    pub domain: String,
    pub db: Connection,
    locals: Locals,
}

impl<'a, T> ServerRequest<'a, T> {
    fn make_context(&self, local_values: Value) -> Value {
        let global_values = context! { env => ENV };
        let request_values = context! { profiles => self.locals.profiles };
        context! { ..local_values, ..request_values, ..global_values }
    }

    pub fn render(&self, path: &str, local_values: Value) -> Vec<u8> {
        let tmpl = self.global.env.get_template(path).unwrap();
        let context = self.make_context(local_values);
        tmpl.render(context).unwrap().into_bytes()
    }

    pub fn render_block(&self, path: &str, block_name: &str, local_values: Value) -> Vec<u8> {
        let tmpl = self.global.env.get_template(path).unwrap();
        let context = self.make_context(local_values);
        let rv = tmpl.eval_to_state(context).unwrap().render_block(block_name).unwrap().into_bytes();
        rv
    }
}

impl<'a, T> Deref for ServerRequest<'a, T> {
    type Target = hyper::Request<T>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl<'a, T> ServerRequest<'a, T> {
    pub fn new(request: hyper::Request<T>, g_ctx: &Arc<GlobalContext<'a>>, db: Connection, domain: String) -> Result<Self, ServerError> {
        let profiles = {
            let mut query = db.prepare("SELECT profile_id, internal_name FROM profiles")?;
            let rows = query.query_map((), |row| {
                let profiles = Profile {
                    profile_id: row.get(0)?,
                    internal_name: row.get(1)?,
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
        Ok(Self { request, global: g_ctx.clone(), db, domain, locals })
    }
}

impl<'a> ServerRequest<'a, Incoming> {
    pub async fn get_body(self) -> Result<ServerRequest<'a, Bytes>, ServerError> {
        let (parts, body) = self.request.into_parts();
        let body_bytes = http_body_util::Limited::new(body, 1024 * 64);

        let bytes = body_bytes.collect().await
            .map(|r| { r.to_bytes() })
            .map_err(|_| { body_too_large() })?;

        let request = hyper::Request::from_parts(parts, bytes);
        let domain = self.domain;
        let global = self.global;
        let db = self.db;
        let locals = self.locals;

        Ok(ServerRequest { request, global, db, domain, locals })
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
        let str = String::from_utf8(body.to_vec()).map_err(|_| { body_not_utf8() })?;
        let request = hyper::Request::from_parts(parts, str);
        let domain = self.domain;
        let global = self.global;
        let db = self.db;
        let locals = self.locals;

        Ok(ServerRequest { request, global, db, domain, locals })
    }
}

impl<'a> ServerRequest<'a, String> {
    pub fn get_form_data<T>(&'a self) -> Result<T, ServerError> where T: Deserialize<'a> {
        let str = self.body();
        serde_html_form::from_str::<T>(str)
            .map_err(|e| {
                warn!("failed to deserialize request {}", &self.body());
                map_bad_request(e)
            })
    }
}