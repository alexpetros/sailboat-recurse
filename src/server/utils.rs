use serde::Deserialize;
use tracing::log::warn;
use crate::server::error::{map_bad_gateway, ServerError};

pub fn deserialize_json<'a, T>(text: &'a str) -> Result<T, ServerError> where T: Deserialize<'a> {
    serde_json::from_str::<T>(text).map_err(|e| {
        warn!("failed to deserialize response {}", text);
        map_bad_gateway(e)
    })
}
