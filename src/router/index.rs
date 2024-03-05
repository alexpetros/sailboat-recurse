use minijinja::context;
use crate::request_utils;
use crate::GlobalContext;
use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub fn get(_req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let tmpl = ctx.env.get_template("index.html").unwrap();
    let context = context! {
        name => "Alex",
        bio => "Rigging my sailboat"
    };
    let body = tmpl.render(context).unwrap().into_bytes();
    Ok(request_utils::send(body))
}
