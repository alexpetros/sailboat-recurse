use serde::Deserialize;
use serde::Serialize;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use crate::server::server_response::redirect;
use crate::server::server_request::IncomingRequest;
use crate::server::server_response::ServerResponse;

pub mod new;
pub mod _profile_id;

#[derive(Serialize, Deserialize)]
struct Profile {
    profile_id: i64,
    handle: String,
    display_name: String,
    internal_name: String,
    private_key_pem: String
}

#[derive(Deserialize)]
struct NewProfile {
    handle: String,
    display_name: String,
    internal_name: String
}

static LONG_ACCEPT_HEADER: &str = "application/ld+json;profile=“https://www.w3.org/ns/activitystreams";
static SHORT_ACCEPT_HEADER: &str = "application/activity+json";

pub async fn post(req: IncomingRequest<'_>) -> ServerResponse {
    let req = req.to_text().await?;
    let form: NewProfile = req.get_form_data()?;

    // TODO encrypt this
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap().private_key_to_pem_pkcs8().unwrap();
    let pkey = String::from_utf8(pkey).unwrap();

    req.db.execute(
        "INSERT INTO profiles (handle, display_name, internal_name, private_key_pem)
        VALUES (?1, ?2, ?3, ?4)",
        (&form.handle, &form.display_name, &form.internal_name, &pkey)
    )?;

    let id = req.db.last_insert_rowid();
    let path = format!("/profiles/{}", id);

    redirect(&path)
}
