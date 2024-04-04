use std::ops::Deref;
use hyper::body::{Bytes, Incoming};
use http_body_util::BodyExt;
use crate::server::error::ServerError;

use super::error::{body_not_utf8, body_too_large};

pub struct IncomingRequest {
    pub request: hyper::Request<Incoming>,
    pub domain: String,
}

impl IncomingRequest {
    pub fn new(request: hyper::Request<Incoming>, domain: String) -> Self {
        Self { request, domain }
    }

    pub async fn get_body(self) -> Result<FullRequest, ServerError> {
        let (parts, body) = self.request.into_parts();
        let body_bytes = http_body_util::Limited::new(body, 1024 * 64);

        let bytes = body_bytes.collect().await
            .map(|r| { r.to_bytes() })
            .map_err(|_| { body_too_large() })?;

        let request = hyper::Request::from_parts(parts, bytes);
        let domain = self.domain;
        Ok(FullRequest { request, domain })
    }
}

impl Deref for IncomingRequest {
    type Target = hyper::Request<Incoming>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

pub struct FullRequest {
    pub request: hyper::Request<Bytes>,
    pub domain: String,
}

impl Deref for FullRequest {
    type Target = hyper::Request<Bytes>;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl FullRequest {
    pub fn text(&self) -> Result<String, ServerError> {
        let body = self.body().to_vec();
        String::from_utf8(body).map_err(|_| { body_not_utf8() })
    }
}
