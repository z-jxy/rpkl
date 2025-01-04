use std::io::{Read, Write};

use serde::Serialize;

use crate::{
    api::evaluator::{
        decode_pkl_message,
        outgoing::{
            codes::{
                CLOSE_EXTERNAL_PROCESS, INITIALIZE_RESOURCE_READER_REQUEST,
                INITIALIZE_RESOURCE_READER_RESPONSE, READ_RESOURCE_REQUEST, READ_RESOURCE_RESPONSE,
            },
            ClientResourceReader,
        },
        responses::{InitializeResourceReaderResponse, PklServerResponseRaw},
    },
    utils::macros::{_info, _warn},
};

#[cfg(feature = "trace")]
use tracing::{info, warn};

use super::{
    outgoing::{ReadResourceError, ReadResourceResponse},
    ExternalReaderClient,
};

#[repr(u64)]
pub enum ReaderCodes {
    // InitializeResourceReaderRequest = 0x30,
    ReadResourceRequest = READ_RESOURCE_REQUEST,
    CloseExternalProcess = CLOSE_EXTERNAL_PROCESS,
    InitializeResourceReaderRequest = INITIALIZE_RESOURCE_READER_REQUEST,
}

impl TryFrom<u64> for ReaderCodes {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            INITIALIZE_RESOURCE_READER_REQUEST => Ok(ReaderCodes::InitializeResourceReaderRequest),
            READ_RESOURCE_REQUEST => Ok(ReaderCodes::ReadResourceRequest),
            CLOSE_EXTERNAL_PROCESS => Ok(ReaderCodes::CloseExternalProcess),
            _ => Err(()),
        }
    }
}

pub struct ExternalReaderRuntime {
    resource_readers: Vec<Box<dyn ExternalReaderClient>>,
}

pub trait IntoReaders {
    fn into_readers(self) -> Vec<Box<dyn ExternalReaderClient>>;
}

macro_rules! impl_into_readers {
    // Base case with one type parameter
    // ($type1:ident) => {
    //     // Already covered by the generic impl above
    // };

    // Recursive case: implement for N type parameters
    ($($type:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($type),+> IntoReaders for ($($type),+)
        where
            $($type: ExternalReaderClient + 'static),+
        {
            fn into_readers(self) -> Vec<Box<dyn ExternalReaderClient>> {
                let ($($type),+) = self;
                vec![$(Box::new($type)),+]
            }
        }
    };
}

impl<T: ExternalReaderClient + 'static> IntoReaders for T {
    fn into_readers(self) -> Vec<Box<dyn ExternalReaderClient>> {
        vec![Box::new(self)]
    }
}

impl_into_readers!(T1, T2);
impl_into_readers!(T1, T2, T3);
impl_into_readers!(T1, T2, T3, T4);
impl_into_readers!(T1, T2, T3, T4, T5);

impl ExternalReaderRuntime {
    pub fn new<T: ExternalReaderClient + 'static>(client: T) -> Self {
        Self {
            resource_readers: vec![Box::new(client)],
        }
    }

    pub fn add_reader<T: ExternalReaderClient + 'static>(&mut self, client: T) {
        self.resource_readers.push(Box::new(client));
    }

    pub fn from(readers: Vec<Box<dyn ExternalReaderClient>>) -> Self {
        Self {
            resource_readers: readers,
        }
    }

    pub fn from_readers<T: IntoReaders>(readers: T) -> Self {
        Self {
            resource_readers: readers.into_readers(),
        }
    }

    fn handle_initalize_reader<R: Read, W: Write>(
        &self,
        pkl_msg: &PklServerResponseRaw,
        r#in: &mut R,
        out: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // let data = rmpv::decode::read_value(r#in)?;

        // _info!("Received data size: {:?}", data);

        // let pkl_msg = decode_pkl_message(data).unwrap();

        if pkl_msg.header != INITIALIZE_RESOURCE_READER_REQUEST {
            _warn!("Invalid message header: {:?}", pkl_msg.header);
            return Err("Invalid message header".into());
        }

        let map = pkl_msg.response.as_map().unwrap();
        let request_id = map.get(0).unwrap().1.as_i64().unwrap();
        let scheme = map.get(1).unwrap().1.as_str().unwrap();

        // TODO: send error to pkl
        let is_compatible = self.resource_readers.iter().any(|r| r.scheme() == scheme);

        if !is_compatible {
            _warn!("Incompatible scheme: {:?}", scheme);
            let msg = InitializeResourceReaderResponse {
                request_id: request_id,
                spec: None,
            };

            let mut serialized = Vec::new();
            (INITIALIZE_RESOURCE_READER_RESPONSE, msg).serialize(
                &mut rmp_serde::Serializer::new(&mut serialized)
                    .with_struct_map()
                    .with_binary(),
            )?;

            out.write_all(&serialized)?;
            out.flush()?;

            return Ok(());
        }

        let msg = InitializeResourceReaderResponse {
            request_id: request_id,
            spec: Some(ClientResourceReader {
                scheme: scheme.to_owned(),
                has_hierarchical_uris: true,
                is_globbable: true,
            }),
        };

        let mut serialized = Vec::new();
        (INITIALIZE_RESOURCE_READER_RESPONSE, msg).serialize(
            &mut rmp_serde::Serializer::new(&mut serialized)
                .with_struct_map()
                .with_binary(),
        )?;

        out.write_all(&serialized)?;
        out.flush()?;

        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for _reader in self.resource_readers.iter() {
            _info!("Registered Reader: {:?}", _reader.scheme());
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
                Ok(ReaderCodes::ReadResourceRequest) => {
                    self.handle_read_resource(&pkl_msg, &mut stdout);
                }
                Ok(ReaderCodes::CloseExternalProcess) => {
                    _info!("CLOSE_EXTERNAL_PROCESS received");
                    break;
                }
                Ok(ReaderCodes::InitializeResourceReaderRequest) => {
                    self.handle_initalize_reader(&pkl_msg, &mut stdin, &mut stdout)?;
                }
                _ => {
                    _warn!("unimplemented message header: {:x}", pkl_msg.header);
                }
            }
        }

        Ok(())
    }

    fn handle_read_resource<W: Write>(&self, msg: &PklServerResponseRaw, writer: &mut W) {
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

        let uri_scheme = extract_scheme(uri).expect("Invalid URI, this is a bug");

        let Some(reader) = self
            .resource_readers
            .iter()
            .find(|r| r.scheme() == uri_scheme)
        else {
            _warn!("No reader found for scheme: {:?}", uri);
            writer
                .write_all(&reader_pack_msg(
                    READ_RESOURCE_RESPONSE,
                    ReadResourceError {
                        request_id,
                        evaluator_id,
                        error: format!("No reader found for scheme: {:?}", uri),
                    },
                ))
                .unwrap();
            writer.flush().unwrap();
            return;
        };

        let data = reader.read(uri);

        let serialized = match data {
            Ok(data) => reader_pack_msg(
                READ_RESOURCE_RESPONSE,
                ReadResourceResponse {
                    request_id,
                    evaluator_id,
                    contents: data,
                },
            ),
            Err(e) => reader_pack_msg(
                READ_RESOURCE_RESPONSE,
                ReadResourceError {
                    request_id,
                    evaluator_id,
                    error: e.to_string(),
                },
            ),
        };

        writer.write_all(&serialized).unwrap();
        writer.flush().unwrap();
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

fn extract_scheme(uri: &str) -> Option<&str> {
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
