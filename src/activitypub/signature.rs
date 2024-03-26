use chrono::DateTime;
use openssl::base64;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::sign::Signer;
use openssl::pkey::Private;
use chrono::Utc;
use hyper::Method;
use openssl::pkey::PKey;

pub fn get_signature_header(method: Method, key_id: &str, target: &str, host: &str, date: DateTime<Utc>, pkey: PKey<Private>) -> Result<String, ErrorStack> {
    let signature = get_signature(method, target, host, date, pkey)?;
    let header = format!(
        r#"keyId="{}",headers="(request-target) host date",signature="{}""#,
        key_id, signature
        );

    Ok(header)
}

pub fn get_signature(method: Method, target: &str, host: &str, date: DateTime<Utc>, pkey: PKey<Private>) -> Result<String, ErrorStack> {
    let method = method.as_str().to_lowercase();
    let date = date.format("%a, %d %b %Y %X GMT");

    let headers = format!("
(request-target): {} {}
host: {}
date: {}",
        method, target, host, date);
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)?;
    signer.update(headers.as_bytes())?;
    let vec = signer.sign_to_vec()?;
    Ok(base64::encode_block(&vec))
 }
