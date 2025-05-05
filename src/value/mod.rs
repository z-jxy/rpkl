pub mod datasize;

#[allow(clippy::module_inception)] // exporting PklValue below
pub mod value;

pub use datasize::DataSize;
pub use value::PklValue;

/// 64-bit signed integer range https://pkl-lang.org/package-docs/pkl/0.26.1/base/IntSeq
pub type IntSeq = std::ops::Range<i64>;
