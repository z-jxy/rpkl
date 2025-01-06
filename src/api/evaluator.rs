use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::{
    api::reader::{
        handle_list_modules, handle_list_resources, handle_read_module, handle_read_resource,
    },
    error::{Error, Result},
    internal::msgapi::{
        incoming::PklServerMessage,
        outgoing::{CloseEvaluator, CreateEvaluator, EvaluateRequest, ExternalReader},
        PklMessage,
    },
    utils::{self, macros::_debug},
};

use crate::internal::msgapi::codes::{
    LIST_MODULES_REQUEST, LIST_RESOURCES_REQUEST, READ_MODULE_REQUEST, READ_RESOURCE_REQUEST,
};

use crate::{api::decoder::pkl_eval_module, pkl::PklMod};

#[cfg(feature = "trace")]
use tracing::debug;

use super::reader::{IntoModuleReaders, IntoResourceReaders, PklModuleReader, PklResourceReader};

pub(crate) const EVALUATE_RESPONSE: u64 = 0x24;
pub(crate) const CREATE_EVALUATOR_REQUEST_ID: u64 = 135;
pub(crate) const OUTGOING_MESSAGE_REQUEST_ID: u64 = 9805131;

// options that can be provided to the evaluator, such as properties (-p flag from CLI)
pub struct EvaluatorOptions {
    /// Properties to pass to the evaluator. Used to read from `props:` in `.pkl` files
    pub properties: Option<HashMap<String, String>>,

    /// Client-side module readers
    pub client_module_readers: Option<Vec<Box<dyn PklModuleReader>>>,

    /// Client-side resource readers
    pub client_resource_readers: Option<Vec<Box<dyn PklResourceReader>>>,

    /// External resource readers
    pub external_resource_readers: Option<HashMap<String, ExternalReader>>,

    /// External module readers
    pub external_module_readers: Option<HashMap<String, ExternalReader>>,
}

impl Default for EvaluatorOptions {
    fn default() -> Self {
        Self {
            properties: None,
            client_module_readers: None,
            client_resource_readers: None,
            external_resource_readers: None,
            external_module_readers: None,
        }
    }
}

impl EvaluatorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a property to the evaluator options map
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let Some(properties) = self.properties.as_mut() {
            properties.insert(key.into(), value.into());
        } else {
            let mut map = HashMap::new();
            map.insert(key.into(), value.into());
            self.properties = Some(map);
        }
        self
    }

    /// Set properties for the evaluator. This will replace any existing properties
    pub fn properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.properties = Some(
            properties
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        );
        self
    }

    /// Add client-side resource readers to the evaluator options map
    pub fn add_client_resource_readers(mut self, readers: impl IntoResourceReaders) -> Self {
        if let Some(vec) = self.client_resource_readers.as_mut() {
            vec.extend(readers.into_readers());
        } else {
            self.client_resource_readers = Some(Vec::from(readers.into_readers()));
        }
        self
    }

    /// Add client module readers to the evaluator options map
    pub fn add_client_module_readers(mut self, readers: impl IntoModuleReaders) -> Self {
        if let Some(vec) = self.client_module_readers.as_mut() {
            vec.extend(readers.into_readers());
        } else {
            self.client_module_readers = Some(Vec::from(readers.into_readers()));
        }
        self
    }

    /// Add an external resource reader to the evaluator options map
    pub fn external_resource_reader(
        mut self,
        key: impl Into<String>,
        executable: impl Into<String>,
        args: &[&str],
    ) -> Self {
        let reader = ExternalReader {
            executable: executable.into(),
            arguments: args
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        };

        if let Some(readers) = self.external_resource_readers.as_mut() {
            readers.insert(key.into(), reader);
        } else {
            let map = HashMap::from([(key.into(), reader)]);
            self.external_resource_readers = Some(map);
        }
        self
    }

    pub fn external_module_reader(
        mut self,
        key: impl Into<String>,
        executable: impl Into<String>,
        args: &[&str],
    ) -> Self {
        let reader = ExternalReader {
            executable: executable.into(),
            arguments: args.into_iter().map(|s| s.to_string()).collect(),
        };

        if let Some(readers) = self.external_module_readers.as_mut() {
            readers.insert(key.into(), reader);
        } else {
            let map = HashMap::from([(key.into(), reader)]);
            self.external_module_readers = Some(map);
        }
        self
    }
}

pub struct Evaluator {
    pub evaluator_id: i64,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
    client_module_readers: Vec<Box<dyn PklModuleReader>>,
    client_resource_readers: Vec<Box<dyn PklResourceReader>>,
}

impl Evaluator {
    pub fn id(&self) -> i64 {
        self.evaluator_id
    }

    pub fn new() -> Result<Self> {
        return Self::new_from_options(EvaluatorOptions::default());
    }

    pub fn new_from_options(options: EvaluatorOptions) -> Result<Self> {
        let mut child = start_pkl(false).map_err(|_e| Error::PklProcessStart)?;
        let child_stdin = child.stdin.as_mut().unwrap();
        let mut child_stdout = child.stdout.take().unwrap();

        let evaluator_message = CreateEvaluator::from(&options);

        let serialized_request = evaluator_message.encode_msg()?;

        let create_eval_response =
            pkl_send_msg_child(child_stdin, &mut child_stdout, serialized_request)
                .map_err(|_e| Error::PklSend)?;

        let Some(map) = create_eval_response.response.as_map() else {
            return Err(Error::PklMalformedResponse {
                message: "expected map in response".to_string(),
            });
        };

        let Some(evaluator_id) = map
            .iter()
            .find(|(k, _v)| k.as_str() == Some("evaluatorId"))
            .and_then(|(_, v)| v.as_i64())
        else {
            return Err(Error::PklMalformedResponse {
                message: "expected evaluatorId in CreateEvaluator response".into(),
            });
        };

        Ok(Evaluator {
            evaluator_id,
            stdin: child.stdin.take().unwrap(),
            stdout: child_stdout,
            client_module_readers: options.client_module_readers.unwrap_or_default(),
            client_resource_readers: options.client_resource_readers.unwrap_or_default(),
        })
    }

    pub fn evaluate_module(&mut self, path: PathBuf) -> Result<PklMod> {
        let evaluator_id = self.id();
        let mut child_stdin = &mut self.stdin;
        let mut child_stdout = &mut self.stdout;

        let path = utils::canonicalize(path)
            .map_err(|_e| Error::Message("failed to canonicalize pkl module path".into()))?;

        let msg = EvaluateRequest {
            request_id: OUTGOING_MESSAGE_REQUEST_ID,
            evaluator_id: evaluator_id,
            module_uri: format!("file://{}", path.to_str().unwrap()),
        }
        .encode_msg()?;

        // send the evaluate request
        child_stdin.write_all(&msg)?;
        child_stdin.flush()?;

        // handle any requests until we get the evaluate response
        let mut eval_res = None;
        while let Ok(msg) = recv_msg(&mut child_stdout) {
            if msg.header == EVALUATE_RESPONSE {
                eval_res = Some(msg);
                break;
            }

            match msg.header {
                READ_RESOURCE_REQUEST => {
                    handle_read_resource(&self.client_resource_readers, &msg, &mut child_stdin)?;
                }
                READ_MODULE_REQUEST => {
                    handle_read_module(&self.client_module_readers, &msg, &mut child_stdin)?;
                }
                LIST_MODULES_REQUEST => {
                    handle_list_modules(&self.client_module_readers, &msg, &mut child_stdin)?;
                }
                LIST_RESOURCES_REQUEST => {
                    handle_list_resources(&self.client_resource_readers, &msg, &mut child_stdin)?;
                }
                _ => {
                    todo!("unhandled request from pkl server: 0x{:x}", msg.header);
                }
            }
        }

        let Some(eval_res) = eval_res else {
            return Err(Error::Message("failed to evaluate pkl module".into()));
        };

        let res = eval_res.response.as_map().unwrap();
        let Some((_, result)) = res.iter().find(|(k, _v)| k.as_str() == Some("result")) else {
            // pkl module evaluation failed, return the error message from pkl
            if let Some((_, error)) = res.iter().find(|(k, _v)| k.as_str() == Some("error")) {
                return Err(Error::PklServerError {
                    pkl_error: error.as_str().unwrap().to_owned(),
                });
            }

            return Err(Error::PklMalformedResponse {
                message: "expected result or error in evaluate response".into(),
            });
        };

        let slice = result.as_slice().unwrap();
        let rmpv_ast: rmpv::Value = rmpv::decode::value::read_value(&mut &slice[..])?;

        _debug!("rmpv pkl module: {:#?}", rmpv_ast);
        let pkl_mod = pkl_eval_module(&rmpv_ast)?;

        Ok(pkl_mod)
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        let mut child_stdin = &mut self.stdin;

        let msg = CloseEvaluator {
            evaluator_id: self.evaluator_id,
        }
        .encode_msg()
        .expect("failed to encode close evaluator message");

        let _ = pkl_send_msg_one_way(&mut child_stdin, msg).expect("failed to close evaluator");
    }
}

pub fn start_pkl(pkl_debug: bool) -> Result<Child> {
    let mut command = Command::new("pkl");

    command
        .arg("server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    if pkl_debug {
        command.env("PKL_DEBUG", "1");
    }

    let child = command.spawn()?;

    Ok(child)
}

pub fn pkl_send_msg_one_way(
    child_stdin: &mut std::process::ChildStdin,

    serialized_request: Vec<u8>,
) -> Result<()> {
    child_stdin.write_all(&serialized_request)?;
    child_stdin.flush()?;
    Ok(())
}

pub fn decode_pkl_message(value: rmpv::Value) -> Option<PklServerMessage> {
    let decoded_array = value.as_array()?;
    let header = decoded_array.first()?.as_u64()?;
    let response = decoded_array.get(1)?.to_owned();
    Some(PklServerMessage { header, response })
}

pub fn pkl_send_msg_child(
    child_stdin: &mut std::process::ChildStdin,
    child_stdout: &mut std::process::ChildStdout,
    serialized_request: Vec<u8>,
) -> Result<PklServerMessage> {
    child_stdin.write_all(&serialized_request)?;
    child_stdin.flush()?;

    // TODO: refactor this to use decode_pkl_message
    match rmpv::decode::read_value(child_stdout) {
        Ok(response) => {
            let decoded_array = response
                .as_array()
                .expect("expected server response to be formatted as an array");
            let first_element = decoded_array.first().expect(
                "malformed server response, received empty array, expected array of length 2",
            );
            let message_header_hex = first_element.as_u64().expect(
                "malformed server response, expected first element to be a u64 representing the message header",
            );
            let second = decoded_array.get(1).expect(
                "malformed server response, expected second element to be a u64 representing the message header",
            );

            return Ok(PklServerMessage {
                header: message_header_hex,
                response: second.to_owned(),
            });
        }
        Err(e) => Err(Error::Message(format!(
            "\nfailed to decode value from pkl process: {:?}",
            e
        ))),
    }
}

pub fn pkl_send_msg_raw(
    child_stdin: &mut impl Write,
    child_stdout: &mut impl Read,
    serialized_request: Vec<u8>,
) -> Result<PklServerMessage> {
    child_stdin.write_all(&serialized_request)?;
    child_stdin.flush()?;

    match rmpv::decode::read_value(child_stdout) {
        Ok(response) => {
            let decoded_array = response
                .as_array()
                .expect("expected server response to be formatted as an array");
            let first_element = decoded_array.first().expect(
                "malformed server response, received empty array, expected array of length 2",
            );
            let message_header_hex = first_element.as_u64().expect(
                "malformed server response, expected first element to be a u64 representing the message header",
            );
            let second = decoded_array.get(1).expect(
                "malformed server response, expected second element to be a u64 representing the message header",
            );

            return Ok(PklServerMessage {
                header: message_header_hex,
                response: second.to_owned(),
            });
        }
        Err(e) => Err(Error::Message(format!(
            "\nfailed to decode value from pkl process: {:?}",
            e
        ))),
    }
}

pub fn recv_msg<R: Read>(reader: &mut R) -> std::result::Result<PklServerMessage, Error> {
    let data = rmpv::decode::read_value(reader)?;
    let pkl_msg = decode_pkl_message(data).ok_or(Error::PklMalformedResponse {
        message: "Failed to decode pkl message format".to_string(),
    })?;
    Ok(pkl_msg)
}
