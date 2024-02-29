use minijinja::context;
use std::sync::Arc;
use crate::request_utils::full;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};
use minijinja::Environment;

pub fn get(_req: Request<Incoming>, env: Arc<Environment<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let tmpl = env.get_template("index.html").unwrap();
    let context = context! {
        name => "Alex",
        bio => "Rigging my sailboat"
    };
    let body = tmpl.render(context).unwrap().into_bytes();

    Ok(Response::new(full(body)))
}
