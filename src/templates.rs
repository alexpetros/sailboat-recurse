use chrono::{DateTime, Local};
use minijinja::{Environment, Error, ErrorKind};

pub mod _partials;

pub fn load_env() -> Environment<'static> {
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);
    env.add_function("unix_time_to_local", unix_time_to_local);
    env
}

// TODO: User-local timestamp handling
fn unix_time_to_local(unix_time: String) -> Result<String, Error> {
    let unix_time = unix_time.parse::<i64>().map_err(|e| {
        Error::new(ErrorKind::InvalidOperation, e.to_string())
    })?;
    let time = DateTime::from_timestamp(unix_time, 0)
        .ok_or(Error::new(ErrorKind::InvalidOperation, "Invalid timestamp provided"))?
        .with_timezone(&Local);
    let local_time = time.format("%b %m %Y, %r").to_string();
    Ok(local_time)
}
