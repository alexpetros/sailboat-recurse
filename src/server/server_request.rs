use std::ops::Deref;
use std::sync::Arc;
use hyper::body::{Bytes, Incoming};
use http_body_util::BodyExt;
use minijinja::{context, Value};
use rusqlite::Connection;
use serde::Serialize;
use crate::server::context::GlobalContext;
use crate::server::error::ServerError;

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

impl<'a> ServerRequest<'a, Bytes> {
    pub fn text(&self) -> Result<String, ServerError> {
        let body = self.body().to_vec();
        String::from_utf8(body).map_err(|_| { body_not_utf8() })
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