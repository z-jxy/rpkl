use std::io::{Read, Write};

use serde::Serialize;

use crate::{
    api::{
        evaluator::{
            decode_pkl_message,
            outgoing::{
                codes::{
                    CLOSE_EXTERNAL_PROCESS, INITIALIZE_MODULE_READER_REQUEST,
                    INITIALIZE_MODULE_READER_RESPONSE, INITIALIZE_RESOURCE_READER_REQUEST,
                    INITIALIZE_RESOURCE_READER_RESPONSE, READ_MODULE_REQUEST,
                    READ_RESOURCE_REQUEST, READ_RESOURCE_RESPONSE,
                },
                ClientModuleReader, ClientResourceReader,
            },
            responses::PklServerResponseRaw,
        },
        external_reader::outgoing::{
            InitializeModuleReaderResponse, InitializeResourceReaderResponse,
        },
        msgapi::PklMessage,
    },
    utils::macros::{_info, _warn},
};

#[cfg(feature = "trace")]
use tracing::{info, warn};

use super::{
    outgoing::{ReadModuleResponse, ReadResourceResponse},
    PklModuleReader, PklResourceReader,
};

/// Codes for the different types of messages that can be sent to the external reader.
#[repr(u64)]
pub enum ReaderCodes {
    // InitializeResourceReaderRequest = 0x30,
    ReadModuleRequest = READ_MODULE_REQUEST,
    ReadResourceRequest = READ_RESOURCE_REQUEST,
    CloseExternalProcess = CLOSE_EXTERNAL_PROCESS,
    InitializeResourceReaderRequest = INITIALIZE_RESOURCE_READER_REQUEST,
    InitializeModuleReaderRequest = INITIALIZE_MODULE_READER_REQUEST,
}

impl TryFrom<u64> for ReaderCodes {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            INITIALIZE_RESOURCE_READER_REQUEST => Ok(ReaderCodes::InitializeResourceReaderRequest),
            INITIALIZE_MODULE_READER_REQUEST => Ok(ReaderCodes::InitializeModuleReaderRequest),
            READ_RESOURCE_REQUEST => Ok(ReaderCodes::ReadResourceRequest),
            READ_MODULE_REQUEST => Ok(ReaderCodes::ReadModuleRequest),
            CLOSE_EXTERNAL_PROCESS => Ok(ReaderCodes::CloseExternalProcess),
            _ => Err(()),
        }
    }
}

pub struct ExternalReaderRuntime {
    resource_readers: Vec<Box<dyn PklResourceReader>>,
    module_readers: Vec<Box<dyn PklModuleReader>>,
}

pub trait IntoResourceReaders {
    fn into_readers(self) -> Vec<Box<dyn PklResourceReader>>;
}

pub trait IntoModuleReaders {
    fn into_readers(self) -> Vec<Box<dyn PklModuleReader>>;
}

// macro_rules! impl_into_readers {
//     ($($type:ident),+) => {
//         #[allow(non_snake_case)]
//         impl<$($type),+> IntoResourceReaders for ($($type),+)
//         where
//             $($type: PklResourceReader + 'static),+
//         {
//             fn into_readers(self) -> Vec<Box<dyn PklResourceReader>> {
//                 let ($($type),+) = self;
//                 vec![$(Box::new($type)),+]
//             }
//         }
//     };
// }

macro_rules! impl_into_readers {
    (resource, $(($type:ident)),+) => {
        #[allow(non_snake_case)]
        impl<$($type),+> IntoResourceReaders for ($($type),+)
        where
            $($type: PklResourceReader + 'static),+
        {
            fn into_readers(self) -> Vec<Box<dyn PklResourceReader>> {
                let ($($type),+) = self;
                vec![$(Box::new($type)),+]
            }
        }
    };
    (module, $(($type:ident)),+) => {
        #[allow(non_snake_case)]
        impl<$($type),+> IntoModuleReaders for ($($type),+)
        where
            $($type: PklModuleReader + 'static),+
        {
            fn into_readers(self) -> Vec<Box<dyn PklModuleReader>> {
                let ($($type),+) = self;
                vec![$(Box::new($type)),+]
            }
        }
    };
}

impl<T: PklResourceReader + 'static> IntoResourceReaders for T {
    fn into_readers(self) -> Vec<Box<dyn PklResourceReader>> {
        vec![Box::new(self)]
    }
}

impl<T: PklModuleReader + 'static> IntoModuleReaders for T {
    fn into_readers(self) -> Vec<Box<dyn PklModuleReader>> {
        vec![Box::new(self)]
    }
}

// impl_into_readers!(T1, T2);
// impl_into_readers!(T1, T2, T3);
// impl_into_readers!(T1, T2, T3, T4);
// impl_into_readers!(T1, T2, T3, T4, T5);

impl_into_readers!(resource, (T1), (T2));
impl_into_readers!(resource, (T1), (T2), (T3));
impl_into_readers!(resource, (T1), (T2), (T3), (T4));
impl_into_readers!(resource, (T1), (T2), (T3), (T4), (T5));

impl_into_readers!(module, (T1), (T2));
impl_into_readers!(module, (T1), (T2), (T3));
impl_into_readers!(module, (T1), (T2), (T3), (T4));
impl_into_readers!(module, (T1), (T2), (T3), (T4), (T5));

impl ExternalReaderRuntime {
    pub fn new() -> Self {
        Self {
            resource_readers: Vec::new(),
            module_readers: Vec::new(),
        }
    }

    pub fn add_resource_readers<T: IntoResourceReaders>(&mut self, readers: T) -> &mut Self {
        self.resource_readers.extend(readers.into_readers());
        self
    }

    pub fn add_module_readers<T: IntoModuleReaders>(&mut self, readers: T) -> &mut Self {
        self.module_readers.extend(readers.into_readers());
        self
    }

    fn handle_initalize_resource_reader<W: Write>(
        &self,
        pkl_msg: &PklServerResponseRaw,
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
        pkl_msg: &PklServerResponseRaw,
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

        let msg = InitializeModuleReaderResponse {
            request_id,
            spec: Some(ClientModuleReader {
                scheme: scheme.to_owned(),
                has_hierarchical_uris: reader.has_hierarchical_uris(),
                is_globbable: reader.is_globbable(),
                is_local: reader.is_local(),
            }),
        };

        let mut serialized = Vec::new();
        (INITIALIZE_MODULE_READER_RESPONSE, msg).serialize(
            &mut rmp_serde::Serializer::new(&mut serialized)
                .with_struct_map()
                .with_binary(),
        )?;

        out.write_all(&serialized)?;
        out.flush()?;

        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();

        for _reader in self.resource_readers.iter() {
            _info!("Registered resource reader: {:?}", _reader.scheme());
        }

        for _reader in self.module_readers.iter() {
            _info!("Registered module reader: {:?}", _reader.scheme());
        }

        loop {
            let Ok(data) = rmpv::decode::read_value(&mut stdin) else {
                _warn!("Failed to read data from stdin");
                break;
            };
            let Some(pkl_msg) = decode_pkl_message(data) else {
                _warn!("Failed to decode message");
                break;
            };

            match ReaderCodes::try_from(pkl_msg.header) {
                Ok(ReaderCodes::InitializeResourceReaderRequest) => {
                    self.handle_initalize_resource_reader(&pkl_msg, &mut stdout)?;
                }
                Ok(ReaderCodes::InitializeModuleReaderRequest) => {
                    self.handle_initalize_module_reader(&pkl_msg, &mut stdout)?;
                }
                Ok(ReaderCodes::ReadResourceRequest) => {
                    self.handle_read_resource(&pkl_msg, &mut stdout)?;
                }
                Ok(ReaderCodes::ReadModuleRequest) => {
                    self.handle_read_module(&pkl_msg, &mut stdout)?;
                }
                Ok(ReaderCodes::CloseExternalProcess) => {
                    _info!("CLOSE_EXTERNAL_PROCESS received");
                    break;
                }
                _ => {
                    _warn!("unimplemented message type: {:x}", pkl_msg.header);
                }
            }
        }

        Ok(())
    }

    fn handle_read_resource<W: Write>(
        &self,
        msg: &PklServerResponseRaw,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = msg.response.as_map().unwrap();

        // TODO: could add `with-serde` feature to rmpv to make this easier
        // but might be overkill for messages with a small number of fields

        let evaluator_id = response
            .iter()
            .find(|(k, _v)| k.as_str() == Some("evaluatorId"))
            .and_then(|(_, v)| v.as_i64())
            .unwrap();

        let request_id = response
            .iter()
            .find(|(k, _v)| k.as_str() == Some("requestId"))
            .and_then(|(_, v)| v.as_i64())
            .unwrap();

        let uri = response
            .iter()
            .find(|(k, _v)| k.as_str() == Some("uri"))
            .and_then(|(_, v)| v.as_str())
            .unwrap();

        let uri_scheme = parse_scheme(uri).expect("Invalid URI, this is a bug");

        let Some(reader) = self
            .resource_readers
            .iter()
            .find(|r| r.scheme() == uri_scheme)
        else {
            _warn!("No reader found for scheme: {:?}", uri);
            writer.write_all(&reader_pack_msg(
                READ_RESOURCE_RESPONSE,
                ReadResourceResponse {
                    request_id,
                    evaluator_id,
                    contents: None,
                    error: Some(format!("No reader found for scheme: {:?}", uri)),
                },
            ))?;
            writer.flush()?;
            return Ok(());
        };

        let data = reader.read(uri);

        let out_msg = match data {
            Ok(data) => ReadResourceResponse {
                request_id,
                evaluator_id,
                contents: Some(data),
                error: None,
            },
            Err(e) => ReadResourceResponse {
                request_id,
                evaluator_id,
                contents: None,
                error: Some(e.to_string()),
            },
        };

        let serialized = reader_pack_msg(READ_RESOURCE_RESPONSE, out_msg);

        writer.write_all(&serialized)?;
        writer.flush()?;

        Ok(())
    }

    fn handle_read_module<W: Write>(
        &self,
        msg: &PklServerResponseRaw,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = msg.response.as_map().unwrap();

        let evaluator_id = response
            .iter()
            .find(|(k, _v)| k.as_str() == Some("evaluatorId"))
            .and_then(|(_, v)| v.as_i64())
            .unwrap();

        let request_id = response
            .iter()
            .find(|(k, _v)| k.as_str() == Some("requestId"))
            .and_then(|(_, v)| v.as_i64())
            .unwrap();

        let uri = response
            .iter()
            .find(|(k, _v)| k.as_str() == Some("uri"))
            .and_then(|(_, v)| v.as_str())
            .unwrap();

        let uri_scheme = parse_scheme(uri).expect("Invalid URI, this is a bug");

        let Some(reader) = self
            .module_readers
            .iter()
            .find(|r| r.scheme() == uri_scheme)
        else {
            _warn!("No reader found for scheme: {:?}", uri);
            writer.write_all(
                &ReadModuleResponse {
                    request_id,
                    evaluator_id,
                    contents: None,
                    error: Some(format!("No reader found for scheme: {:?}", uri)),
                }
                .encode_msg()?,
            )?;
            writer.flush()?;
            return Ok(());
        };

        let data = reader.read(uri);

        let out_msg = match data {
            Ok(data) => ReadModuleResponse {
                request_id,
                evaluator_id,
                contents: Some(data),
                error: None,
            },
            Err(e) => ReadModuleResponse {
                request_id,
                evaluator_id,
                contents: None,
                error: Some(e.to_string()),
            },
        };

        let serialized = out_msg.encode_msg()?;

        writer.write_all(&serialized)?;
        writer.flush()?;

        Ok(())
    }
}

/// uses `.with_bytes(rmp_serde::config::BytesMode::ForceAll)` to serialize bytes
fn reader_pack_msg<T: Serialize>(header: u64, msg: T) -> Vec<u8> {
    let mut serialized_request = Vec::new();
    let _ = &(header, msg)
        .serialize(
            &mut rmp_serde::Serializer::new(&mut serialized_request)
                .with_struct_map()
                .with_bytes(rmp_serde::config::BytesMode::ForceAll),
        )
        .unwrap();
    serialized_request
}

fn parse_scheme(uri: &str) -> Option<&str> {
    match uri.find(':') {
        Some(pos) => {
            let scheme = &uri[..pos];
            if !scheme.is_empty()
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
            {
                Some(scheme)
            } else {
                None
            }
        }
        None => None,
    }
}
