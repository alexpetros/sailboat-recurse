use std::str::FromStr;

use chrono::{DateTime, Local};
use minijinja::{Environment, Error};
use tracing::warn;

pub mod _partials;

pub fn load_env() -> Environment<'static> {
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);
    env.add_function("iso_to_local", iso_to_local);
    env
}

fn iso_to_local(iso_time: String) -> Result<String, Error> {
    let time = DateTime::<Local>::from_str(&iso_time);
    match time {
        Ok(t) => {
            let local_time = t.format("%b %m %Y, %r").to_string();
            Ok(local_time)
        }
        Err(_) => {
            warn!(
                "Invalid time {}; time should be in ISO 8601 format",
                &iso_time
            );
            Ok(iso_time)
        }
    }
}
