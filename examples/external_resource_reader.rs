use std::io::{Read, Write};

use rpkl::api::{
    self,
    evaluator::{
        self, decode_pkl_message,
        outgoing::{ClientResourceReader, OutgoingMessage},
        pkl_send_msg_child, pkl_send_msg_raw, ExternalReader,
    },
    external_reader::{reader::ExternalReaderRuntime, ExternalReaderClient},
};

use serde::{Deserialize, Serialize};

pub struct Reader;

impl ExternalReaderClient for Reader {
    // const READER_TYPE: api::external_reader::ReaderType =
    //     api::external_reader::ReaderType::Resource;

    fn reader_type(&self) -> api::external_reader::ReaderType {
        api::external_reader::ReaderType::Resource
    }

    fn read(&self, uri: &str) -> Option<Vec<u8>> {
        Some("hello world".as_bytes().to_vec())
    }
}

#[cfg(feature = "trace")]
use tracing::{error, info, warn};
// use tracing_subscriber::filter::EnvFilter;
#[cfg(feature = "trace")]
use tracing_subscriber::{
    fmt, fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn main() {
    #[cfg(feature = "trace")]
    {
        // Create a log file
        let log_file = std::fs::File::create("output.log").unwrap();

        // Set up a tracing subscriber
        let file_layer = fmt::layer()
            .with_writer(log_file) // Write logs to the file
            .with_ansi(false); // Disable ANSI colors for file logs

        // Use environment variables to set log levels, or default to `info`
        // let filter_layer =
        //     EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(file_layer)
            // .with(filter_layer)
            .init();
    }

    ExternalReaderRuntime::new(Reader).run();
}
