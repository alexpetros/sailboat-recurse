use crate::request;
use crate::request::global_context::GlobalContext;
use minijinja::context;
use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub fn get(_req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let context = context! {
        posts => vec! [
            context! { user_name => "Alex", user_handle => "awp@alexpetros.com" }
        ],
        name => "Alex",
        bio => "Rigging my sailboat"
    };
    let body = ctx.render("index.html", context);
    Ok(request::send(body))
}
