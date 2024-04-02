use std::time::SystemTime;
use minijinja::Value;
use minijinja::context;
use serde::Serialize;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use minijinja::Environment;
use std::sync::Arc;
use rusqlite::Connection;

use super::error::ServerError;

const ENV: &str = if cfg!(debug_assertions) { "debug" } else { "prod" };

#[derive(Clone)]
pub struct GlobalContext<'a> {
    pub env: Arc<Environment<'a>>,
    pub statics: Arc<HashMap<String, Vec<u8>>>,
    pub startup_time: u128,
    pub domain: Option<String>
}

impl <'a>GlobalContext<'a> {
    pub fn new(env: Arc<Environment<'a>>, statics: Arc<HashMap<String, Vec<u8>>>) -> GlobalContext {
        let startup_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        GlobalContext { env, statics, startup_time, domain: None }
    }
}

#[derive(Serialize)]
struct Feed {
    feed_id: i64,
    internal_name: String
}

#[derive(Serialize)]
struct Locals {
    feeds: Vec<Feed>
}

pub struct Context<'a> {
    pub global: Arc<GlobalContext<'a>>,
    pub db: Connection,
    locals: Locals
}

impl<'a> Context<'a> {
    pub fn new (g_ctx: &Arc<GlobalContext<'a>>, db: Connection) -> Result<Context<'a>, ServerError> {
        let feeds = {
            let mut query = db.prepare("SELECT feed_id, internal_name FROM feeds")?;
            let rows = query.query_map((), |row| {
                let feed = Feed {
                    feed_id: row.get(0)?,
                    internal_name: row.get(1)?,
                };
                Ok(feed)
            })?;

            let mut feeds = Vec::new();
            for feed in rows {
                feeds.push(feed?);
            }

            feeds
        };

        let locals = Locals { feeds };
        let context = Context { global: g_ctx.clone(), db, locals };
        Ok(context)
    }

    fn make_context(&self, local_values: Value) -> Value {
        let global_values = context! { env => ENV };
        let request_values = context! { feeds => self.locals.feeds };
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


