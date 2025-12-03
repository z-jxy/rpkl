mod utils;

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use rpkl::api::evaluator::EvaluatorOptions;
    use rpkl::{HttpOptions, HttpProxy};
    use serde::Deserialize;

    #[test]
    fn optional_values() {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Config {
            ip: Option<String>,
            port: u16,
            database: Database,
            pet_name: Option<String>,
        }

        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Database {
            username: String,
            password: String,
        }

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("database.pkl");
        let config = rpkl::from_config::<Config>(path).unwrap();

        assert_eq!(config.pet_name, Some("Doggo".into()));
    }

    // [pkl is failing to read `env:` resource on windows](https://github.com/apple/pkl/issues/1077)
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn resources() {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Config {
            path: String,
            name: String,
        }

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("allowed-resources.pkl");

        let options = EvaluatorOptions::default().properties([("name", "zjxy")]);

        let config = rpkl::from_config_with_options::<Config>(path, options).unwrap();

        assert_eq!(config.name, "zjxy");
    }

    // #[test]
    // TODO: figure out a better way to run this test
    #[allow(dead_code)]
    pub fn external_resource_readers() {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Config {
            username: String,
            ldap_email: String,
            ldaps_email: String,
        }

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("external-reader.pkl");

        let options = EvaluatorOptions::default()
            .external_resource_reader(
                "ldap",
                "target/debug/examples/external_resource_reader",
                &[],
            )
            .external_resource_reader(
                "ldaps",
                "target/debug/examples/external_resource_reader",
                &[],
            )
            .external_module_reader(
                "remote",
                "target/debug/examples/external_resource_reader",
                &[],
            );

        rpkl::from_config_with_options::<Config>(path, options).unwrap();
    }

    #[test]
    fn http_proxy_options_builder() {
        // Test that the builder API works correctly
        let proxy = HttpProxy::new("http://proxy.example.com:8080")
            .no_proxy(["localhost", "127.0.0.1", "10.0.0.0/8"]);

        assert_eq!(
            proxy.address,
            Some("http://proxy.example.com:8080".to_string())
        );
        assert_eq!(
            proxy.no_proxy,
            Some(vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "10.0.0.0/8".to_string()
            ])
        );
    }

    #[test]
    fn http_options_builder() {
        // Test building HttpOptions with proxy
        let http = HttpOptions::new()
            .proxy(HttpProxy::new("http://proxy.example.com:8080"))
            .ca_certificates(vec![1, 2, 3, 4]);

        assert!(http.proxy.is_some());
        assert_eq!(http.ca_certificates, Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn evaluator_options_with_http() {
        // Test that EvaluatorOptions can be built with HTTP config
        let options = EvaluatorOptions::new()
            .http(
                HttpOptions::new().proxy(
                    HttpProxy::new("http://proxy.example.com:8080")
                        .no_proxy(["localhost", "*.internal.net"]),
                ),
            )
            .timeout_seconds(30)
            .property("key", "value");

        assert!(options.http.is_some());
        assert_eq!(options.timeout_seconds, Some(30));
        assert!(options.properties.is_some());

        let http = options.http.as_ref().unwrap();
        let proxy = http.proxy.as_ref().unwrap();
        assert_eq!(
            proxy.address,
            Some("http://proxy.example.com:8080".to_string())
        );
        assert_eq!(
            proxy.no_proxy,
            Some(vec![
                "localhost".to_string(),
                "*.internal.net".to_string()
            ])
        );
    }
}

#[cfg(test)]
mod non_primitive_values {
    use crate::utils::pkl_tests_file;
    use std::path::PathBuf;

    #[derive(serde::Deserialize, Debug)]
    #[allow(dead_code)]
    pub struct Config {
        duration: std::time::Duration,
        size: rpkl::value::DataSize,
        pair: (i32, i32),
        range: std::ops::Range<i64>,
        #[serde(rename(deserialize = "emailRegex"))]
        email_regex: String,
        #[serde(rename(deserialize = "intList"))]
        int_list: Vec<i32>,

        pair2: (Vec<i32>, Vec<i32>),
        numbers: Vec<rpkl::Value>,
    }

    #[test]
    fn non_primitive_values() -> Result<(), rpkl::Error> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("nonprim.pkl");
        let config = rpkl::from_config::<Config>(path)?;

        assert!(config.duration.as_millis() == 12);

        assert!(config.pair2.0 == vec![1, 2, 3] && config.pair2.1 == vec![4, 5, 6]);

        assert!(config.range.start == 2 && config.range.end == 5);

        assert!(config.numbers.len() == 4);

        assert!(!config.numbers[0].is_number());

        assert!(config.numbers[2].is_number());

        assert!(config.numbers[3].is_bool());

        Ok(())
    }

    #[test]
    fn mappings() -> Result<(), rpkl::Error> {
        #[derive(serde::Deserialize, Debug)]
        struct MappingConfig {
            paths: std::collections::HashMap<String, Vec<String>>,
        }

        let path = pkl_tests_file("mappings.pkl");

        let config = rpkl::from_config::<MappingConfig>(path)?;

        assert!(config.paths.len() == 1);

        let val = config.paths.get("*").unwrap();

        assert!(val.len() == 2);

        Ok(())
    }

    #[test]
    fn bytes() {
        #[derive(serde::Deserialize, Debug)]
        struct Config {
            my_bytes: Vec<u8>,
        }

        let config = rpkl::from_config::<Config>(pkl_tests_file("bytes.pkl"))
            .expect("deserialize bytes.pkl");

        assert!(
            config.my_bytes
                == vec![
                    0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21
                ]
        );

        #[derive(serde::Deserialize, Debug)]
        struct BytesConfig {
            my_bytes: rpkl::Value,
        }

        let bytes_config = rpkl::from_config::<BytesConfig>(pkl_tests_file("bytes.pkl"))
            .expect("deserialize bytes.pkl");

        assert!(bytes_config.my_bytes.is_bytes());
        assert_eq!(&bytes_config.my_bytes.as_bytes().unwrap(), &config.my_bytes);
    }
}
