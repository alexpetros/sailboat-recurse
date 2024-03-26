use openssl::rsa::Padding;
use tracing::log::debug;
use chrono_tz::Tz;
use chrono::DateTime;
use openssl::base64;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::sign::Signer;
use openssl::pkey::Private;
use hyper::Method;
use openssl::pkey::PKey;

pub fn get_signature_header(method: Method, key_id: &str, target: &str, host: &str, date: DateTime<Tz>, pkey: PKey<Private>) -> Result<String, ErrorStack> {
    let signature = get_signature(method, target, host, date, pkey)?;
    let header = format!(
        r#"keyId="{}",headers="(request-target) host date",signature="{}""#,
        key_id, signature
        );

    Ok(header)
}

pub fn get_signature(method: Method, target: &str, host: &str, date: DateTime<Tz>, pkey: PKey<Private>) -> Result<String, ErrorStack> {
    let method = method.as_str().to_lowercase();
    let date = date.format("%a, %d %b %Y %X %Z");

    let headers = format!("
(request-target): {} {}
host: {}
date: {}",
        method, target, host, date);
    debug!("HEADERS:\n{}", headers);
    debug!("PKEY:\n{}", String::from_utf8(pkey.private_key_to_pem_pkcs8().unwrap()).unwrap());
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)?;
    signer.set_rsa_padding(Padding::PKCS1).unwrap();
    debug!("PADDING:\n{:?}", signer.rsa_padding().unwrap());
    signer.update(headers.as_bytes())?;
    let vec = signer.sign_to_vec()?;
    Ok(base64::encode_block(&vec))
 }
