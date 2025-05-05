#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use rmpv::Value;
    use rpkl::api;
    use rpkl::api::evaluator::EvaluatorOptions;
    use rpkl::pkl::Deserializer;
    use rpkl::pkl::PklSerialize;
    use serde::Deserialize;
    use test::Bencher;

    #[cfg(feature = "dhat-heap")]
    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

    macro_rules! print_time {
        ($elapsed:expr) => {
            if $elapsed.as_millis() > 0 {
                println!("Time: {}ms", $elapsed.as_millis());
            } else {
                println!("Time: {}μs", $elapsed.as_micros());
            }
        };
    }

    /// Time a block of code
    macro_rules! time {
        ( $($s:stmt);* $(;)?) => {
            let now = std::time::Instant::now();
            {
                $($s)*
            };
            let elapsed = now.elapsed();
            print_time!(elapsed);
        }
    }

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
    fn deserialize_time() {
        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();

        #[derive(Debug, PartialEq, Deserialize)]
        struct Config {
            ip: String,
            port: u16,
            birds: Vec<String>,
            database: Database,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct Database {
            username: String,
            password: String,
        }

        let ast = Value::Array(vec![
            Value::Integer(1.into()),
            Value::String("example".into()),
            Value::String("file:///Users/testing/code/rust/rpkl/examples/example.pkl".into()),
            Value::Array(vec![
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("ip".into()),
                    Value::String("127.0.0.1".into()),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("port".into()),
                    Value::Integer(8080.into()),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("birds".into()),
                    Value::Array(vec![
                        Value::Integer(5.into()),
                        Value::Array(vec![
                            Value::String("Pigeon".into()),
                            Value::String("Hawk".into()),
                            Value::String("Penguin".into()),
                        ]),
                    ]),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("database".into()),
                    Value::Array(vec![
                        Value::Integer(1.into()),
                        Value::String("Dynamic".into()),
                        Value::String("pkl:base".into()),
                        Value::Array(vec![
                            Value::Array(vec![
                                Value::Integer(16.into()),
                                Value::String("username".into()),
                                Value::String("admin".into()),
                            ]),
                            Value::Array(vec![
                                Value::Integer(16.into()),
                                Value::String("password".into()),
                                Value::String("secret".into()),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]);
        let expected = Config {
            ip: "127.0.0.1".into(),
            port: 8080,
            birds: vec!["Pigeon".into(), "Hawk".into(), "Penguin".into()],
            database: Database {
                username: "admin".to_owned(),
                password: "secret".to_owned(),
            },
        };
        let now = std::time::Instant::now();
        let pkl_mod = api::pkl_eval_module(&ast).expect("failed to evaluate pkl ast");
        let mapped = pkl_mod
            .serialize_pkl_ast()
            .expect("failed to serialize pkl module");

        let deserialized = Config::deserialize(&mut Deserializer::from_pkl_map(&mapped))
            .expect("failed to deserialize");

        let elapsed = now.elapsed();
        print_time!(elapsed);
        assert_eq!(expected, deserialized);
    }

    #[bench]
    fn deserialize(b: &mut Bencher) {
        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();

        #[derive(Debug, PartialEq, Deserialize)]
        struct Config {
            ip: String,
            port: u16,
            birds: Vec<String>,
            database: Database,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct Database {
            username: String,
            password: String,
        }

        let ast = Value::Array(vec![
            Value::Integer(1.into()),
            Value::String("example".into()),
            Value::String("file:///Users/testing/code/rust/rpkl/examples/example.pkl".into()),
            Value::Array(vec![
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("ip".into()),
                    Value::String("127.0.0.1".into()),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("port".into()),
                    Value::Integer(8080.into()),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("birds".into()),
                    Value::Array(vec![
                        Value::Integer(5.into()),
                        Value::Array(vec![
                            Value::String("Pigeon".into()),
                            Value::String("Hawk".into()),
                            Value::String("Penguin".into()),
                        ]),
                    ]),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("database".into()),
                    Value::Array(vec![
                        Value::Integer(1.into()),
                        Value::String("Dynamic".into()),
                        Value::String("pkl:base".into()),
                        Value::Array(vec![
                            Value::Array(vec![
                                Value::Integer(16.into()),
                                Value::String("username".into()),
                                Value::String("admin".into()),
                            ]),
                            Value::Array(vec![
                                Value::Integer(16.into()),
                                Value::String("password".into()),
                                Value::String("secret".into()),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]);

        time! {
            b.iter(|| {
                for _ in 0..100 {
                    let pkl_mod = api::pkl_eval_module(&ast).expect("failed to evaluate pkl ast");
                    let mapped = pkl_mod
                        .serialize_pkl_ast()
                        .expect("failed to serialize pkl module");

                    Config::deserialize(&mut Deserializer::from_pkl_map(& mapped))
                        .expect("failed to deserialize");
                }
            });
        }
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
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("mappings.pkl");

        #[derive(serde::Deserialize, Debug)]
        struct MappingConfig {
            paths: std::collections::HashMap<String, Vec<String>>,
        }

        let config = rpkl::from_config::<MappingConfig>(path)?;

        assert!(config.paths.len() == 1);

        let val = config.paths.get("*").unwrap();

        assert!(val.len() == 2);

        Ok(())
    }
}
