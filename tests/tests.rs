#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use rpkl::api::evaluator::EvaluatorOptions;
    use serde::Deserialize;

    #[cfg(feature = "dhat-heap")]
    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

    #[test]
    fn optional_values() {
        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();

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

    #[test]
    fn resources() {
        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();

        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Config {
            path: String,
            name: String,
            // package: rpkl::Value,
        }

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("allowed-resources.pkl");

        let options = EvaluatorOptions::default().properties([("name", "zjxy")]);

        let config = rpkl::from_config_with_options::<Config>(path, options).unwrap();

        assert_eq!(config.name, "zjxy");
    }

    #[test]
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
}

#[cfg(test)]
mod non_primitive_values {
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

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("mappings.pkl");

        let config = rpkl::from_config::<MappingConfig>(path)?;

        assert!(config.paths.len() == 1);

        let val = config.paths.get("*").unwrap();

        assert!(val.len() == 2);

        Ok(())
    }
}
