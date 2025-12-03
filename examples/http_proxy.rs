//! Example demonstrating HTTP proxy configuration for pkl evaluation.
//!
//! This is useful when you need to fetch remote pkl packages through a corporate proxy.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example http_proxy
//! ```

use rpkl::{EvaluatorOptions, HttpOptions, HttpProxy};
use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ip: Option<String>,
    port: u16,
    database: Database,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Database {
    username: String,
    password: String,
}

fn main() {
    // Example 1: Simple proxy configuration
    let _simple_proxy = EvaluatorOptions::new()
        .http(HttpOptions::new().proxy(HttpProxy::new("http://proxy.example.com:8080")));

    // Example 2: Proxy with no_proxy list for bypassing certain hosts
    let _proxy_with_bypass = EvaluatorOptions::new().http(
        HttpOptions::new().proxy(
            HttpProxy::new("http://proxy.example.com:8080")
                .no_proxy(["localhost", "127.0.0.1", "*.internal.company.com", "10.0.0.0/8"]),
        ),
    );

    // Example 3: Full configuration with proxy, timeout, and properties
    let options = EvaluatorOptions::new()
        .http(
            HttpOptions::new().proxy(
                HttpProxy::new("http://proxy.example.com:8080")
                    .no_proxy(["localhost", "127.0.0.1"]),
            ),
        )
        .timeout_seconds(60)
        .property("environment", "production");

    println!("Configured evaluator options with HTTP proxy:");
    println!("  - Proxy: {:?}", options.http.as_ref().and_then(|h| h.proxy.as_ref()).and_then(|p| p.address.as_ref()));
    println!("  - No-proxy: {:?}", options.http.as_ref().and_then(|h| h.proxy.as_ref()).and_then(|p| p.no_proxy.as_ref()));
    println!("  - Timeout: {:?} seconds", options.timeout_seconds);

    // Actually evaluate a local pkl file (no proxy needed for local files)
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join("database.pkl");

    // Use default options for local file (proxy only needed for remote packages)
    let config: Config = rpkl::from_config(&path).unwrap();
    println!("\nLoaded config: {config:?}");
}
