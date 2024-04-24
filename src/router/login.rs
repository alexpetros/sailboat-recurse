use hyper::header::{HeaderValue, SET_COOKIE};
use minijinja::context;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;

use crate::server::{server_request::{AnyRequest, PlainRequest}, server_response::{redirect, send, ServerResult}, utils::make_cookie};
use crate::server::server_request::AuthState;

pub async fn get<'a, Au: AuthState>(req: AnyRequest<'a, Au>) -> ServerResult {
    let body = req.render("login.html", context! {})?;
    Ok(send(body))
}

#[derive(Deserialize)]
struct FormData {
    password: String
}

pub async fn post(req: PlainRequest<'_>) -> ServerResult {
    let req = req.to_text().await?;
    let form: FormData = req.get_form_data()?;
    let _pass = form.password;
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    req.db.execute("INSERT INTO sessions (token) VALUES (?1)", (&token,))?;

    let mut res = redirect("/")?;
    let cookie = make_cookie("token", &token);
    res.headers_mut().append(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(res)
}
