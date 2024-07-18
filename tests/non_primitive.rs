#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn non_primitive_values() -> Result<(), rpkl::Error> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("pkl")
            .join("nonprim.pkl");
        let config = rpkl::from_config::<Config>(path)?;

        // println!("{:?}", config);

        assert!(config.duration.as_millis() == 12);

        assert!(config.pair2.0 == vec![1, 2, 3] && config.pair2.1 == vec![4, 5, 6]);

        assert!(config.range.start == 2 && config.range.end == 5);

        Ok(())
    }
}
