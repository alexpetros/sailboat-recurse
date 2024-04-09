use crate::queries::get_posts_in_feed;
use minijinja::context;
use crate::server::request::IncomingRequest;
use crate::server::response;
use crate::server::response::ResponseResult;

pub async fn get(req: IncomingRequest<'_>) -> ResponseResult {
    let posts = get_posts_in_feed(&req.db, 1)?;
    let mut query = req.db.prepare("SELECT count(*) FROM followed_actors")?;
    let follow_count: i64 = query.query_row((), |row| { row.get(0) })?;

    let context = context! {
        posts,
        feed_id => "1",
        name => "Alex",
        follow_count,
    };

    let body = req.render("index.html", context);
    Ok(response::send(body))
}
