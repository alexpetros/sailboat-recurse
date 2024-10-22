use hyper::header::InvalidHeaderValue;
use openssl::error::ErrorStack;
use std::{error::Error, fmt::Display};

use hyper::StatusCode;

#[derive(Debug)]
pub struct ServerError {
    pub prefix: &'static str,
    pub message: String,
    pub status_code: StatusCode,
}

impl PartialEq for ServerError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.status_code == other.status_code
    }
}

// pub trait PartialEq<Rhs = Self>
// where
//     Rhs: ?Sized,
// {
//     fn eq(&self, other: &Rhs) -> bool;
// }

impl Error for ServerError {}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.prefix, self.message)
    }
}

impl From<InvalidHeaderValue> for ServerError {
    fn from(err: InvalidHeaderValue) -> Self {
        ServerError {
            prefix: "[HYPER ERROR]",
            message: err.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<rusqlite::Error> for ServerError {
    fn from(err: rusqlite::Error) -> Self {
        ServerError {
            prefix: "[SQL ERROR]",
            message: err.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> Self {
        ServerError {
            prefix: "[HYPER ERROR]",
            message: err.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ErrorStack> for ServerError {
    fn from(err: ErrorStack) -> Self {
        ServerError {
            prefix: "[OpenSSL ERROR]",
            message: err.to_string(),
            status_code: StatusCode::BAD_REQUEST,
        }
    }
}

pub fn map_bad_request(e: impl Error) -> ServerError {
    ServerError {
        prefix: "[BAD REQUEST]",
        message: format!("{:?}", e),
        status_code: StatusCode::BAD_REQUEST,
    }
}

pub fn map_bad_gateway(e: impl Error) -> ServerError {
    ServerError {
        prefix: "[BAD GATEWAY]",
        message: format!("{:?}", e),
        status_code: StatusCode::BAD_GATEWAY,
    }
}

pub fn not_found() -> ServerError {
    ServerError {
        prefix: "",
        message: "".to_string(),
        status_code: StatusCode::NOT_FOUND,
    }
}

pub fn forbidden() -> ServerError {
    ServerError {
        prefix: "",
        message: "".to_string(),
        status_code: StatusCode::FORBIDDEN
    }
}

pub fn bad_request(message: &str) -> ServerError {
    ServerError {
        prefix: "[BAD REQUEST]",
        message: message.to_owned(),
        status_code: StatusCode::BAD_REQUEST,
    }
}

pub fn body_too_large() -> ServerError {
    ServerError {
        prefix: "",
        message: "Body was too large".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    }
}

pub fn body_not_utf8() -> ServerError {
    ServerError {
        prefix: "",
        message: "Body was not UTF8".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    }
}

pub fn bad_gateway(message: &str) -> ServerError {
    ServerError {
        prefix: "[BAD REQUEST]",
        message: message.to_owned(),
        status_code: StatusCode::BAD_REQUEST,
    }
}
