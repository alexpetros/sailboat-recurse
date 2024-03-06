use std::time::SystemTime;
use minijinja::Value;
use minijinja::context;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use minijinja::Environment;
use std::sync::Arc;

const ENV: &str = if cfg!(debug_assertions) { "debug" } else { "prod" };

#[derive(Clone)]
pub struct GlobalContext<'a> {
    pub env: Arc<Environment<'a>>,
    pub statics: Arc<HashMap<String, Vec<u8>>>,
    pub startup_time: u128
}

impl <'a>GlobalContext<'a> {
    pub fn new(env: Arc<Environment<'a>>, statics: Arc<HashMap<String, Vec<u8>>>) -> GlobalContext {
        let startup_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        GlobalContext { env, statics, startup_time }
    }

    pub fn render(&self, path: &str, local_values: Value) -> Vec<u8> {
        let tmpl = self.env.get_template(path).unwrap();
        let global_values = context! {
            env => ENV
        };
        let context = context! { ..local_values, ..global_values };
        tmpl.render(context).unwrap().into_bytes()
    }
}


