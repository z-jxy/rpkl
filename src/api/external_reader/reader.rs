use std::io::Write;

use serde::Serialize;

use crate::api::evaluator::{
    decode_pkl_message,
    outgoing::{
        codes::{
            INITIALIZE_RESOURCE_READER_REQUEST, INITIALIZE_RESOURCE_READER_RESPONSE,
            READ_RESOURCE_RESPONSE,
        },
        ClientResourceReader,
    },
    pkl_send_msg_raw,
    responses::InitializeResourceReaderResponse,
};

use super::{outgoing::ReadResourceResponse, ExternalReaderClient};

pub struct ExternalReaderRuntime {
    client: Box<dyn ExternalReaderClient>,
}

impl ExternalReaderRuntime {
    pub fn new<T: ExternalReaderClient + 'static>(client: T) -> Self {
        Self {
            client: Box::new(client),
        }
    }

    pub fn run(&self) {
        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let data = rmpv::decode::read_value(&mut stdin).unwrap();

        #[cfg(feature = "trace")]
        {
            info!("Received data size: {:?}", data);
        }

        let pkl_msg = decode_pkl_message(data).unwrap();

        if pkl_msg.header != INITIALIZE_RESOURCE_READER_REQUEST {
            #[cfg(feature = "trace")]
            {
                warn!("Invalid message header: {:?}", pkl_msg.header);
            }
            return;
        }

        let map = pkl_msg.response.as_map().unwrap();

        let request_id = map.get(0).unwrap().1.as_i64().unwrap();
        let scheme = map.get(1).unwrap().1.as_str().unwrap();

        #[cfg(feature = "trace")]
        {
            info!("Request ID: {:?}", request_id);
            info!("Scheme: {:?}", scheme);
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
        let _ = &(INITIALIZE_RESOURCE_READER_RESPONSE, msg)
            .serialize(
                &mut rmp_serde::Serializer::new(&mut serialized)
                    .with_struct_map()
                    .with_binary(),
            )
            .unwrap();

        let response = pkl_send_msg_raw(&mut stdout, &mut stdin, serialized).unwrap();

        let response = response.response.as_map().unwrap();

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

        let data = self.client.read(uri);

        let resource_bin = rmpv::Value::Binary(data.unwrap().clone());
        let res = resource_bin.as_slice().unwrap();

        let msg = ReadResourceResponse {
            evaluator_id: evaluator_id,
            request_id: request_id,
            contents: res.to_vec(),
            // error: None,
        };

        let mut serialized = Vec::new();
        let _ = &(READ_RESOURCE_RESPONSE, msg)
            .serialize(
                &mut rmp_serde::Serializer::new(&mut serialized)
                    .with_struct_map()
                    .with_bytes(rmp_serde::config::BytesMode::ForceAll),
                // .with_binary(),
                // .with_bytes(rmp_serde::config::BytesMode::ForceAll),
            )
            .unwrap();

        stdout.write_all(&serialized).unwrap();
        stdout.flush().unwrap();
    }
}
