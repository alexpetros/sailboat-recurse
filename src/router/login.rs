use chrono::Utc;
use hyper::header::{HeaderValue, SET_COOKIE};
use minijinja::context;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;

use crate::server::{server_request::{UnauthedRequest, SR}, server_response::{redirect, send, ServerResponse}, utils::make_cookie};

pub fn get<T>(req: SR<T>) -> ServerResponse {
    let body = req.render("login.html", context! {})?;
    Ok(send(body))
}

#[derive(Deserialize)]
struct FormData {
    password: String
}

pub async fn post(req: UnauthedRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
    let form: FormData = req.get_form_data()?;
    let _pass = form.password;
    let now = Utc::now().format("%FT%TZ").to_string();
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    req.db.execute("INSERT INTO sessions (token, timestamp) VALUES (?1, ?2)", (&token, now))?;

    let mut res = redirect("/")?;
    let cookie = make_cookie("token", &token);
    res.headers_mut().append(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(res)
}
