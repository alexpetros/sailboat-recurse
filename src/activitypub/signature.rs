use hyper::header::HeaderValue;
use openssl::rsa::Padding;
use chrono_tz::Tz;
use chrono::DateTime;
use openssl::base64;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::sign::Signer;
use openssl::pkey::Private;
use hyper::{Method, Uri};
use openssl::pkey::PKey;

use crate::server::error::ServerError;


pub fn get_signature_header(method: &Method, key_id: &str, uri: &Uri, date: DateTime<Tz>, pkey: &PKey<Private>) -> Result<HeaderValue, ServerError> {

    let signature = get_signature(method, uri, date, pkey)?;
    let header_str = format!(
        r#"keyId="{}",headers="(request-target) host date",signature="{}""#,
        key_id, signature
        );

    let header = HeaderValue::from_str(&header_str).unwrap();
    Ok(header)
}

fn get_signature(method: &Method, uri: &Uri, date: DateTime<Tz>, pkey: &PKey<Private>) -> Result<String, ErrorStack> {
    let method = method.as_str().to_lowercase();
    let date = date.format("%a, %d %b %Y %X %Z");
    let host = uri.host().unwrap();
    let target = uri.path_and_query().unwrap();

    let headers = format!("(request-target): {} {}\nhost: {}\ndate: {}", method, target, host, date);
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)?;
    signer.set_rsa_padding(Padding::PKCS1).unwrap();
    signer.update(headers.as_bytes())?;
    let vec = signer.sign_to_vec()?;
    Ok(base64::encode_block(&vec))
 }
