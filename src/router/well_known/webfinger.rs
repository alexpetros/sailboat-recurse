use crate::server::context::Context;
use crate::server::request::Request;
use crate::server::response;
use crate::server::response::ResponseResult;

pub async fn get(_req: Request, _ctx: Context<'_>) -> ResponseResult {
    // let body = req.get_body().await?;
    // let text = body.text()?;
    Ok(response::send(""))
}
