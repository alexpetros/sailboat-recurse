use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use minijinja::Environment;

#[derive(Clone)]
pub struct GlobalContext<'a> {
    pub env: Arc<Environment<'a>>,
    pub statics: Arc<HashMap<String, Vec<u8>>>,
    pub startup_time: u128,
    pub domain: Option<String>
}

impl <'a>GlobalContext<'a> {
    pub fn new(env: Arc<Environment<'a>>, statics: Arc<HashMap<String, Vec<u8>>>) -> GlobalContext {
        let startup_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        GlobalContext { env, statics, startup_time, domain: None }
    }
}

