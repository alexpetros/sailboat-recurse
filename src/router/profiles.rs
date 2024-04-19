use crate::server::server_request::UnauthedRequest;
use crate::server::server_response::redirect;
use crate::server::server_response::ServerResponse;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use serde::Deserialize;
use serde::Serialize;

pub mod _profile_id;
pub mod new;

#[derive(Serialize, Deserialize)]
struct Profile {
    profile_id: i64,
    preferred_username: String,
    display_name: String,
    nickname: String,
    private_key_pem: String,
}

#[derive(Deserialize)]
struct NewProfile {
    preferred_username: String,
    display_name: String,
    nickname: String,
}

static LONG_ACCEPT_HEADER: &str =
    "application/ld+json;profile=“https://www.w3.org/ns/activitystreams";
static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

pub async fn post(req: UnauthedRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
    let form: NewProfile = req.get_form_data()?;

    // TODO encrypt this
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa)
        .unwrap()
        .private_key_to_pem_pkcs8()
        .unwrap();
    let pkey = String::from_utf8(pkey).unwrap();

    req.db.execute(
        "INSERT INTO profiles (preferred_username, display_name, nickname, private_key_pem)
        VALUES (?1, ?2, ?3, ?4)",
        (
            &form.preferred_username,
            &form.display_name,
            &form.nickname,
            &pkey,
        ),
    )?;

    let id = req.db.last_insert_rowid();
    let path = format!("/profiles/{}", id);

    redirect(&path)
}
