#![rustfmt::skip]
/* Generated by rpkl */

#[derive(Debug, ::serde::Deserialize)]
pub struct Example {
	#[serde(rename = "ip")]
	pub ip: String,
	pub port: i64,
	pub ints: std::ops::Range<i64>,
	pub birds: Vec<rpkl::Value>,
	pub mapping: rpkl::Value,
	pub anon_map: example::AnonMap,
	pub database: example::Database,
	pub mode: example::Mode,
}


pub mod example {
	#[derive(Debug, ::serde::Deserialize)]
	#[derive(Default)]
	pub struct AnonMap {
		pub anon_key: String,
		#[serde(rename = "anon_key2")]
		pub anon_key_2: String,
	}
	
	#[derive(Debug, ::serde::Deserialize)]
	pub struct Database {
		pub username: String,
		pub password: String,
	}
	
	#[derive(Debug, ::serde::Deserialize)]
	#[derive(Default)]
	pub enum Mode {
		#[default]
		Dev,
		Production,
	}
	
}