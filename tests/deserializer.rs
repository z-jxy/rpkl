#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use api::deserializer::Deserializer;
    use pkl_rs::api;
    use pkl_rs::pkl::PklSerialize;
    use rmpv::Value;
    use serde::Deserialize;
    use test::Bencher;

    #[cfg(feature = "dhat-heap")]
    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

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
            Value::String("file:///Users/testing/code/rust/pkl-rs/examples/example.pkl".into()),
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
        b.iter(|| {
            for _ in 0..100 {
                let pkl_mod = api::pkl_eval_module(&ast).expect("failed to evaluate pkl ast");
                let mut mapped = pkl_mod
                    .serialize_pkl_ast()
                    .expect("failed to serialize pkl module");

                Config::deserialize(&mut Deserializer::from_pkl_map(&mut mapped))
                    .expect("failed to deserialize");
            }
        });
    }
}
