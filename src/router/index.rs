use crate::request::ResponseResult;
use serde::Serialize;
use crate::request;
use crate::request::global_context::GlobalContext;
use minijinja::context;
use rusqlite::Connection;
use std::sync::Arc;
use hyper::body::Incoming;
use hyper::Request;

#[derive(Debug, Serialize)]
struct Post {
    author_name: String,
    author_handle: String,
    content: String
}

pub fn get(_req: Request<Incoming>, ctx: Arc<GlobalContext<'_>>) -> ResponseResult {
    let conn = Connection::open("./sailboat.db").unwrap();
    let mut query = conn.prepare("SELECT author_name, author_handle, content FROM posts").unwrap();
    let rows = query.query_map((), |row| {
        let post = Post {
            author_name: row.get(0)?,
            author_handle: row.get(1)?,
            content: row.get(2)?
        };
        Ok(post)
    })?;

    let mut posts = Vec::new();
    for post in rows {
        posts.push(post.unwrap())
    }

    // let posts = posts.map(|post| {
    //     let post = post.unwrap();
    //     context! { post }
    // });

    let context = context! {
        // posts => vec! [ context! { user_name => "Alex", user_handle => "awp@alexpetros.com" } ],
        posts,
        name => "Alex",
        bio => "Rigging my sailboat"
    };

    let body = ctx.render("index.html", context);
    Ok(request::send(body))
}
