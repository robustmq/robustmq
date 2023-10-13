#[cfg(test)]
mod tests {

    use toml::Table;
    #[test]
    fn test_toml_table_parser() {
        let value = "foo = 'bar'".parse::<Table>().unwrap();

        assert_eq!(value["foo"].as_str(), Some("bar"));
    }

    #[test]
    fn test_toml_deserialize_parser() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Config {
            ip: String,
            port: Option<u16>,
            keys: Keys,
        }

        #[derive(Deserialize)]
        struct Keys {
            github: String,
            travis: Option<String>,
        }

        let config: Config = toml::from_str(
            r#"
           ip = '127.0.0.1'
        
           [keys]
           github = 'xxxxxxxxxxxxxxxxx'
           travis = 'yyyyyyyyyyyyyyyyy'
        "#,
        )
        .unwrap();

        println!("ip is : {}", config.ip);
        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.port, None);
        assert_eq!(config.keys.github, "xxxxxxxxxxxxxxxxx");
        assert_eq!(config.keys.travis.as_ref().unwrap(), "yyyyyyyyyyyyyyyyy");
    }

    #[test]
    fn test_toml_file_parser() {
        use serde::Deserialize;
        use std::fs::File;
        use std::io::Read;

        #[derive(Deserialize)]
        struct ServerConfig {
            broker: Server,
            admin: Server,
        }

        #[derive(Deserialize)]
        struct Server {
            addr: String,
            port: Option<u16>,
        }

        let file_path = "config/server.toml";
        let mut file = File::open(file_path).expect("Failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");

        // At this point, `contents` contains the content of the TOML file
        println!("{}", contents);

        let server: ServerConfig = toml::from_str(&contents).unwrap();

        println!("ip is : {}", server.broker.addr);
        assert_eq!(server.broker.addr, "127.0.0.1");
        assert_eq!(server.broker.port, Some(1226u16));

        println!("ip is : {}", server.admin.addr);
        assert_eq!(server.admin.addr, "127.0.0.2");
        assert_eq!(server.admin.port, Some(1227u16));
        // assert_eq!(config.keys.github, "xxxxxxxxxxxxxxxxx");
        // assert_eq!(config.keys.travis.as_ref().unwrap(), "yyyyyyyyyyyyyyyyy");
    }
}
