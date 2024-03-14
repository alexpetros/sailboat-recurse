use serde::Serialize;
use crate::server::context::Context;
use minijinja::context;
use crate::server::request::Request;
use crate::server::response;
use crate::server::response::ResponseResult;

#[derive(Debug, Serialize)]
struct Post {
    author_name: String,
    author_handle: String,
    content: String
}

pub fn get(_req: Request, ctx: Context<'_>) -> ResponseResult {
    let mut query = ctx.db.prepare("SELECT author_name, author_handle, content FROM posts")?;
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
