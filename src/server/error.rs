use std::fmt::Display;

#[derive(Debug)]
pub enum ServerError {
    Hyper(hyper::Error),
    Sql(rusqlite::Error),
    BodyTooLarge(),
    BodyNotUtf8()
}

impl std::error::Error for ServerError {}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ServerError::Hyper(ref err) => write!(f, "[HYPER ERROR] {}", err),
            ServerError::Sql(ref err) => write!(f, "[SQL ERROR] {}", err),
            ServerError::BodyTooLarge() => write!(f, "Body Too Large Error"),
            ServerError::BodyNotUtf8() => write!(f, "Body was expected to be UT8, and it wasn't"),
        }
    }
}

impl From<rusqlite::Error> for ServerError {
    fn from(err: rusqlite::Error) -> Self {
        ServerError::Sql(err)
    }
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> Self {
        ServerError::Hyper(err)
    }
}
