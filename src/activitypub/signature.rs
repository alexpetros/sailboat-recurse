use chrono::DateTime;
use chrono_tz::Tz;
use hyper::header::HeaderValue;
use hyper::{Method, Uri};
use openssl::base64;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::pkey::Private;
use openssl::rsa::Padding;
use openssl::sign::Signer;

use crate::server::error::ServerError;

pub fn get_signature_header(
    method: &Method,
    key_id: &str,
    uri: &Uri,
    date: DateTime<Tz>,
    pkey: &PKey<Private>,
    digest: Option<String>
) -> Result<HeaderValue, ServerError> {
    let digest_str = if digest.is_some() { " digest" } else { "" };
    let signature = get_signature(method, uri, date, pkey, digest)?;
    let header_str = format!(
        r#"keyId="{}",headers="(request-target) host date{}",signature="{}""#,
        key_id, digest_str, signature,
    );

    let header = HeaderValue::from_str(&header_str).unwrap();
    Ok(header)
}

fn get_signature(
    method: &Method,
    uri: &Uri,
    date: DateTime<Tz>,
    pkey: &PKey<Private>,
    digest: Option<String>
) -> Result<String, ErrorStack> {
    let method = method.as_str().to_lowercase();
    let date = date.format("%a, %d %b %Y %X %Z");
    let host = uri.host().unwrap();
    let target = uri.path_and_query().unwrap();

    let digest = match digest {
        Some(d) => format!("\ndigest: {}", d),
        None => "".to_owned()
    };

    let headers = format!(
        "(request-target): {} {}\nhost: {}\ndate: {}{}",
        method, target, host, date, digest
    );
    let mut signer = Signer::new(MessageDigest::sha256(), pkey)?;
    signer.set_rsa_padding(Padding::PKCS1).unwrap();
    signer.update(headers.as_bytes())?;
    let vec = signer.sign_to_vec()?;
    Ok(base64::encode_block(&vec))
}
