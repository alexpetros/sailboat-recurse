use serde::Serialize;
use crate::server::context::Context;
use minijinja::context;
use crate::server::request::{GET, Request};
use crate::server::response;
use crate::server::response::{not_found, ResponseResult};

#[derive(Debug, Serialize)]
struct Post {
    post_id: i64,
    author_name: String,
    author_handle: String,
    content: String
}

pub fn router(req: Request, ctx: Context<'_>) -> ResponseResult {
    if req.method() != GET { return not_found(req, ctx) }

    let mut query = ctx.db.prepare("SELECT post_id, author_name, author_handle, content FROM posts")?;
    let rows = query.query_map((), |row| {
        let post = Post {
            post_id: row.get(0)?,
            author_name: row.get(1)?,
            author_handle: row.get(2)?,
            content: row.get(3)?
        };
        Ok(post)
    })?;

    let mut posts = Vec::new();
    for post in rows {
        posts.push(post?)
    }

    let context = context! {
        posts,
        name => "Alex",
        bio => "Rigging my sailboat"
    };

    let body = ctx.render("index.html", context);
    Ok(response::send(body))
}
