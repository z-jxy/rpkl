use std::path::{Path, PathBuf};

pub fn pkl_tests_file<P: AsRef<Path>>(path: P) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join(path)
}
