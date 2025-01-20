use std::io::Write;

use crate::internal::msgapi::{
    codes::{
        CLOSE_EXTERNAL_PROCESS, INITIALIZE_MODULE_READER_REQUEST,
        INITIALIZE_RESOURCE_READER_REQUEST, LIST_MODULES_REQUEST, LIST_RESOURCES_REQUEST,
        READ_MODULE_REQUEST, READ_RESOURCE_REQUEST,
    },
    incoming::PklServerMessage,
    outgoing::{
        ClientModuleReader, ClientResourceReader, InitializeModuleReaderResponse,
        InitializeResourceReaderResponse,
    },
    PklMessage,
};

use crate::{
    api::{
        evaluator::recv_msg,
        reader::{
            handle_list_modules, handle_list_resources, handle_read_module, handle_read_resource,
        },
    },
    utils::macros::{_info, _warn},
};

pub struct ExternalReaderRuntime {
    resource_readers: Vec<Box<dyn PklResourceReader>>,
    module_readers: Vec<Box<dyn PklModuleReader>>,
}

impl ExternalReaderRuntime {
    pub fn new() -> Self {
        Self {
            resource_readers: Vec::new(),
            module_readers: Vec::new(),
        }
    }

    /// Add a single, or tuple of resource readers to the client.
    ///
    /// Panics if any of the readers have the same scheme.
    pub fn add_resource_readers<T: IntoResourceReaders>(&mut self, readers: T) -> &mut Self {
        let readers = readers.into_readers();
        self.resource_readers.extend(readers);

        for (i, reader) in self.resource_readers.iter().enumerate() {
            for other in &self.resource_readers[i + 1..] {
                if reader.scheme() == other.scheme() {
                    panic!(
                        "Multiple resource readers sharing the same scheme: {}",
                        reader.scheme()
                    );
                }
            }
        }

        self
    }

    pub fn add_module_readers<T: IntoModuleReaders>(&mut self, readers: T) -> &mut Self {
        let readers = readers.into_readers();
        self.module_readers.extend(readers);

        for (i, reader) in self.module_readers.iter().enumerate() {
            for other in &self.module_readers[i + 1..] {
                if reader.scheme() == other.scheme() {
                    panic!(
                        "Multiple resource readers sharing the same scheme: {}",
                        reader.scheme()
                    );
                }
            }
        }

        self
    }

    fn handle_initalize_resource_reader<W: Write>(
        &self,
        pkl_msg: &PklServerMessage,
        out: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug_assert!(pkl_msg.header == INITIALIZE_RESOURCE_READER_REQUEST);

        let map = pkl_msg.response.as_map().unwrap();
        let request_id = map.get(0).unwrap().1.as_i64().unwrap();
        let scheme = map.get(1).unwrap().1.as_str().unwrap();

        // TODO: send error to pkl
        let Some(reader) = self.resource_readers.iter().find(|r| r.scheme() == scheme) else {
            _warn!("Incompatible scheme: {:?}", scheme);

            let serialized = InitializeResourceReaderResponse {
                request_id,
                spec: None,
            }
            .encode_msg()?;

            out.write_all(&serialized)?;
            out.flush()?;

            return Ok(());
        };

        let serialized = InitializeResourceReaderResponse {
            request_id,
            spec: Some(ClientResourceReader {
                scheme: scheme.to_owned(),
                has_hierarchical_uris: reader.has_hierarchical_uris(),
                is_globbable: reader.is_globbable(),
            }),
        }
        .encode_msg()?;

        out.write_all(&serialized)?;
        out.flush()?;

        Ok(())
    }

    fn handle_initalize_module_reader<W: Write>(
        &self,
        pkl_msg: &PklServerMessage,
        out: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug_assert!(pkl_msg.header == INITIALIZE_MODULE_READER_REQUEST);

        let map = pkl_msg.response.as_map().unwrap();
        let request_id = map.get(0).unwrap().1.as_i64().unwrap();
        let scheme = map.get(1).unwrap().1.as_str().unwrap();

        // TODO: send error to pkl
        let Some(reader) = self.module_readers.iter().find(|r| r.scheme() == scheme) else {
            _warn!("Incompatible scheme: {:?}", scheme);

            let serialized = InitializeModuleReaderResponse {
                request_id,
                spec: None,
            }
            .encode_msg()?;

            out.write_all(&serialized)?;
            out.flush()?;

            return Ok(());
        };

        let serialized = InitializeModuleReaderResponse {
            request_id,
            spec: Some(ClientModuleReader {
                scheme: scheme.to_owned(),
                has_hierarchical_uris: reader.has_hierarchical_uris(),
                is_globbable: reader.is_globbable(),
                is_local: reader.is_local(),
            }),
        }
        .encode_msg()?;

        out.write_all(&serialized)?;
        out.flush()?;

        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "trace")]
        {
            use tracing_subscriber::{
                fmt, fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
            };
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

        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();

        for _reader in self.resource_readers.iter() {
            _info!("Registered resource reader: {:?}", _reader.scheme());
        }

        for _reader in self.module_readers.iter() {
            _info!("Registered module reader: {:?}", _reader.scheme());
        }

        loop {
            let Ok(pkl_msg) = recv_msg(&mut stdin) else {
                _warn!("Failed to decode message");
                break;
            };

            match pkl_msg.header {
                INITIALIZE_RESOURCE_READER_REQUEST => {
                    self.handle_initalize_resource_reader(&pkl_msg, &mut stdout)?;
                }
                INITIALIZE_MODULE_READER_REQUEST => {
                    self.handle_initalize_module_reader(&pkl_msg, &mut stdout)?;
                }
                LIST_RESOURCES_REQUEST => {
                    handle_list_resources(&self.resource_readers, &pkl_msg, &mut stdout)?;
                }
                LIST_MODULES_REQUEST => {
                    handle_list_modules(&self.module_readers, &pkl_msg, &mut stdout)?;
                }
                READ_RESOURCE_REQUEST => {
                    handle_read_resource(&self.resource_readers, &pkl_msg, &mut stdout)?;
                }
                READ_MODULE_REQUEST => {
                    handle_read_module(&self.module_readers, &pkl_msg, &mut stdout)?;
                }
                CLOSE_EXTERNAL_PROCESS => {
                    _info!("CLOSE_EXTERNAL_PROCESS received");
                    break;
                }
                _ => {
                    _warn!("unexpected message type: {:x}", pkl_msg.header);
                }
            }
        }

        Ok(())
    }
}

pub use crate::api::reader::{PklModuleReader, PklResourceReader};
pub use crate::internal::msgapi::outgoing::PathElements;

use super::reader::{IntoModuleReaders, IntoResourceReaders};
