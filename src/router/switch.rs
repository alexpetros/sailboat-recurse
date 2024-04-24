use crate::server::server_request::PlainRequest;
use crate::server::server_response::{redirect, ServerResult};
use crate::server::utils::make_cookie;
use hyper::header::SET_COOKIE;
use hyper::http::HeaderValue;

pub async fn get(req: PlainRequest<'_>) -> ServerResult {
    let profile_id = req.get_trailing_param("Missing profileId in url")?;
    let mut res = redirect("/")?;
    let cookie = make_cookie("current_profile", profile_id);
    res.headers_mut()
        .append(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(res)
}
