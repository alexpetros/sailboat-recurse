use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::sign::Signer;
use openssl::pkey::Private;
use chrono::Utc;
use hyper::Method;
use openssl::pkey::PKey;

pub fn get_signature(method: Method, target: &str, host: &str, pkey: PKey<Private>) -> Result<Vec<u8>, ErrorStack> {
    let method = method.as_str().to_lowercase();
    let date = Utc::now();
    let headers = format!("
(request-target): {} {}
host: {}
date: {}",
        method, target, host, date);
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)?;
    signer.update(headers.as_bytes())?;
    signer.sign_to_vec()
}
