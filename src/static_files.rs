use std::collections::HashMap;

pub fn load_static() -> HashMap<String, Vec<u8>> {
    let mut static_files = HashMap::new();

    static_files.insert(String::from("common.css"), include_bytes!("./static/common.css").to_vec());
    static_files.insert(String::from("hello.js"), include_bytes!("./static/hello.js").to_vec());
    static_files.insert(String::from("images/favicon.png"), include_bytes!("./static/images/favicon.png").to_vec());
    static_files
}

