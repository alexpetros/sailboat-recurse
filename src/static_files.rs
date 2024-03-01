use std::collections::HashMap;

pub fn load_static() -> HashMap<String, Vec<u8>> {
    let mut scores = HashMap::new();

    scores.insert(String::from("common.css"), include_bytes!("./static/common.css").to_vec());
    scores.insert(String::from("test.js"), include_bytes!("./static/test.js").to_vec());
    scores
}

