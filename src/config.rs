const PORT_DEFAULT: u16 = 3000;

pub struct Config {
    pub port: u16
}

impl Config {
    pub fn new(mut args: Vec<String>) -> Config {

        let port_string = find_flag_with_value(&mut args, "--port");
        let port = match port_string {
            None => PORT_DEFAULT,
            Some(port_string) => {
                port_string.parse()
                    .unwrap_or_else(|_| panic!("Invalid value for port: {}", port_string))
            }
        };
        Config { port }
    }
}

fn find_flag_with_value(args: &mut Vec<String>, flag: &'static str) -> Option<String> {
    args.iter()
        .position(|x| x == flag)
        .map(|index| {
            if index + 1 == args.len() { panic!("Missing value for {}", flag); }
            args.drain(index..index+2).nth(1).unwrap()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! str_vec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn no_args_default() {
        let args = str_vec!["sailboat"];
        let config = Config::new(args);
        assert_eq!(config.port, 3000)
    }

    #[test]
    fn sets_port_with_long_opt() {
        let args = str_vec!["sailboat", "--port", "8080"];
        let config = Config::new(args);
        assert_eq!(config.port, 8080)
    }

    #[test]
    #[should_panic(expected = "Missing value for --port")]
    fn missing_value_after_port() {
        let args = str_vec!["sailboat", "--port"];
        Config::new(args);
    }

    #[test]
    #[should_panic(expected = "Invalid value for port: --other")]
    fn invalid_value_for_port() {
        let args = str_vec!["sailboat", "--port", "--other"];
        Config::new(args);
    }

}
