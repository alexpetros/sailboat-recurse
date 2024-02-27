use crate::request_utils::full;
use http_body_util::{combinators::BoxBody};
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

use minijinja::{Environment, context};

pub fn get(_req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let mut env = Environment::new();
    env.add_template("hello", "Hello {{ name }}!").unwrap();
    let tmpl = env.get_template("hello").unwrap();
    let body = tmpl.render(context!(name => "World")).unwrap().into_bytes();

    Ok(Response::new(full(body)))
}
