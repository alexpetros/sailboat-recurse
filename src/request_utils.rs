use std::time::SystemTime;
use minijinja::Value;
use minijinja::context;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use minijinja::Environment;
use std::sync::Arc;
use hyper::StatusCode;
use hyper::Response;
use http_body_util::Empty;
use http_body_util::Full;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Bytes;

const ENV: &str = if cfg!(debug_assertions) { "debug" } else { "prod" };

#[derive(Clone)]
pub struct GlobalContext<'a> {
    pub env: Arc<Environment<'a>>,
    pub statics: Arc<HashMap<String, Vec<u8>>>,
    pub startup_time: u128
}

impl <'a>GlobalContext<'a> {
    pub fn new(env: Arc<Environment<'a>>, statics: Arc<HashMap<String, Vec<u8>>>) -> GlobalContext {
        let startup_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        GlobalContext { env, statics, startup_time }
    }

    pub fn render(&self, path: &str, local_values: Value) -> Vec<u8> {
        let tmpl = self.env.get_template(path).unwrap();
        let global_values = context! {
            env => ENV
        };
        let context = context! { ..local_values, ..global_values };
        tmpl.render(context).unwrap().into_bytes()
    }
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}

pub fn send<T: Into<Bytes>>(body: T) -> Response<BoxBody<hyper::body::Bytes, hyper::Error>> {
    Response::new(full(body))
}

pub fn not_found () -> Response<BoxBody<hyper::body::Bytes, hyper::Error>> {
    let mut not_found = Response::new(empty());
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    not_found
}

