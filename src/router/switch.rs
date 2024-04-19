use crate::server::server_request::UnauthedRequest;
use crate::server::server_response::{redirect, ServerResponse};
use crate::server::utils::make_cookie;
use hyper::header::SET_COOKIE;
use hyper::http::HeaderValue;

pub fn get(req: UnauthedRequest<'_>) -> ServerResponse {
    let profile_id = req.get_trailing_param("Missing profileId in url")?;
    let mut res = redirect("/")?;
    let cookie = make_cookie("current_profile", profile_id);
    res.headers_mut()
        .append(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(res)
}
