use std::{
    collections::HashMap,
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::Arc,
};

#[cfg(feature = "indexmap")]
use indexmap::IndexMap;

use crate::{
    api::reader::{
        handle_list_modules, handle_list_resources, handle_read_module, handle_read_resource,
    },
    context::Context,
    error::{Error, Result},
    internal::msgapi::{
        PklMessage,
        incoming::PklServerMessage,
        outgoing::{CloseEvaluator, CreateEvaluator, EvaluateRequest, ExternalReader},
    },
    utils::{self, macros::_debug},
    value::value::MapImpl,
};

use crate::internal::msgapi::codes::{
    LIST_MODULES_REQUEST, LIST_RESOURCES_REQUEST, READ_MODULE_REQUEST, READ_RESOURCE_REQUEST,
};

use crate::{decoder::decode_module, pkl::PklMod};

use super::reader::{IntoModuleReaders, IntoResourceReaders, PklModuleReader, PklResourceReader};

pub(crate) const EVALUATE_RESPONSE: u64 = 0x24;
pub(crate) const CREATE_EVALUATOR_REQUEST_ID: u64 = 135;
pub(crate) const OUTGOING_MESSAGE_REQUEST_ID: u64 = 9805131;

/// HTTP proxy configuration for outgoing requests.
///
/// Only HTTP proxies are supported (not HTTPS or SOCKS).
/// The proxy address must use the `http://` scheme and contain only host and port.
#[derive(Default, Clone, Debug)]
pub struct HttpProxy {
    /// The proxy server address (e.g., "http://proxy.example.com:8080").
    /// Must use the `http://` scheme and contain only host and port.
    pub address: Option<String>,

    /// Hosts that should bypass the proxy.
    /// Supports hostnames, IP addresses, and CIDR notation.
    pub no_proxy: Option<Vec<String>>,
}

impl HttpProxy {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: Some(address.into()),
            no_proxy: None,
        }
    }

    /// Set the proxy address.
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Add hosts that should bypass the proxy.
    pub fn no_proxy(mut self, hosts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.no_proxy = Some(hosts.into_iter().map(Into::into).collect());
        self
    }
}

/// HTTP configuration for outgoing requests.
///
/// Added in Pkl 0.26.0.
#[derive(Default, Clone, Debug)]
pub struct HttpOptions {
    /// HTTP proxy configuration.
    pub proxy: Option<HttpProxy>,

    /// PEM-format CA certificates to trust for HTTPS connections.
    pub ca_certificates: Option<Vec<u8>>,
}

impl HttpOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HTTP proxy configuration.
    pub fn proxy(mut self, proxy: HttpProxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Set CA certificates (PEM format) for HTTPS connections.
    pub fn ca_certificates(mut self, certs: Vec<u8>) -> Self {
        self.ca_certificates = Some(certs);
        self
    }
}

// options that can be provided to the evaluator, such as properties (-p flag from CLI)
#[derive(Default, Clone)]
pub struct EvaluatorOptions {
    /// Properties to pass to the evaluator. Used to read from `props:` in `.pkl` files
    pub properties: Option<MapImpl<String, String>>,

    /// Client-side module readers
    pub client_module_readers: Option<Vec<Arc<dyn PklModuleReader>>>,

    /// Client-side resource readers
    pub client_resource_readers: Option<Vec<Arc<dyn PklResourceReader>>>,

    /// External resource readers
    pub external_resource_readers: Option<HashMap<String, ExternalReader>>,

    /// External module readers
    pub external_module_readers: Option<HashMap<String, ExternalReader>>,

    /// HTTP configuration for outgoing requests (proxy, CA certificates).
    /// Added in Pkl 0.26.0.
    pub http: Option<HttpOptions>,

    /// Timeout in seconds for evaluating a source module.
    pub timeout_seconds: Option<u64>,
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
            #[cfg(feature = "indexmap")]
            let mut map = IndexMap::new();
            #[cfg(not(feature = "indexmap"))]
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
                .collect::<MapImpl<_, _>>(),
        );
        self
    }

    /// Add client-side resource readers to the evaluator options map
    pub fn add_client_resource_readers(mut self, readers: impl IntoResourceReaders) -> Self {
        if let Some(vec) = self.client_resource_readers.as_mut() {
            vec.extend(readers.into_readers());
        } else {
            self.client_resource_readers = Some(readers.into_readers());
        }
        self
    }

    /// Add client module readers to the evaluator options map
    pub fn add_client_module_readers(mut self, readers: impl IntoModuleReaders) -> Self {
        if let Some(vec) = self.client_module_readers.as_mut() {
            vec.extend(readers.into_readers());
        } else {
            self.client_module_readers = Some(readers.into_readers());
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
                .iter()
                .map(ToString::to_string)
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
            arguments: args.iter().map(ToString::to_string).collect(),
        };

        if let Some(readers) = self.external_module_readers.as_mut() {
            readers.insert(key.into(), reader);
        } else {
            let map = HashMap::from([(key.into(), reader)]);
            self.external_module_readers = Some(map);
        }
        self
    }

    /// Set HTTP configuration for outgoing requests.
    ///
    /// This allows configuring proxy settings and CA certificates for HTTPS connections.
    /// Added in Pkl 0.26.0.
    ///
    /// # Example
    /// ```no_run
    /// use rpkl::{EvaluatorOptions, HttpOptions, HttpProxy};
    ///
    /// let options = EvaluatorOptions::new()
    ///     .http(HttpOptions::new()
    ///         .proxy(HttpProxy::new("http://proxy.example.com:8080")
    ///             .no_proxy(["localhost", "127.0.0.1"])));
    /// ```
    pub fn http(mut self, http: HttpOptions) -> Self {
        self.http = Some(http);
        self
    }

    /// Set the timeout in seconds for evaluating a source module.
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
}

pub struct Evaluator {
    pub evaluator_id: i64,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
    client_module_readers: Vec<Arc<dyn PklModuleReader>>,
    client_resource_readers: Vec<Arc<dyn PklResourceReader>>,
}

impl Evaluator {
    pub fn id(&self) -> i64 {
        self.evaluator_id
    }

    /// Create a new evaluator with default options
    /// This will start a new pkl process and create an evaluator
    /// The evaluator will be closed when it is dropped
    /// # Errors
    /// - Returns an error if the pkl process fails to start or if the evaluator fails to be created
    pub fn new() -> Result<Self> {
        Self::new_from_options(EvaluatorOptions::default())
    }

    /// Create a new evaluator with the given options
    /// This will start a new pkl process and create an evaluator
    /// The evaluator will be closed when it is dropped
    /// # Errors
    /// - Returns an error if the pkl process fails to start or if the evaluator fails to be created
    pub fn new_from_options(options: EvaluatorOptions) -> Result<Self> {
        let mut child = start_pkl(false).map_err(|_e| Error::PklProcessStart)?;
        let child_stdin = child.stdin.as_mut().context("failed to get stdin")?;
        let mut child_stdout = child.stdout.take().context("failed to get stdout")?;

        let evaluator_message = CreateEvaluator::from(&options);

        let serialized_request = evaluator_message.encode_msg()?;

        let create_eval_response =
            pkl_send_msg_child(child_stdin, &mut child_stdout, &serialized_request)
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
            stdin: child.stdin.take().context("failed to get stdin")?,
            stdout: child_stdout,
            client_module_readers: options.client_module_readers.unwrap_or_default(),
            client_resource_readers: options.client_resource_readers.unwrap_or_default(),
        })
    }

    /// Evaluate a pkl module
    /// This will send the module to the pkl process and return the result
    /// # Errors
    /// - Returns an error if the pkl process fails to evaluate the module or if the module is malformed
    /// - If the provided path is not does not exist
    pub fn evaluate_module(&mut self, path: impl AsRef<std::path::Path>) -> Result<PklMod> {
        let evaluator_id = self.id();
        let mut child_stdin = &mut self.stdin;
        let mut child_stdout = &mut self.stdout;

        let path = utils::canonicalize(path)
            .map_err(|_e| Error::Message("failed to canonicalize pkl module path".into()))?;

        let msg = EvaluateRequest {
            request_id: OUTGOING_MESSAGE_REQUEST_ID,
            evaluator_id,
            module_uri: format!(
                "file://{}",
                path.to_str().context("Path is not valid utf8")?
            ),
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
                    unimplemented!("unimplemented request from pkl server: 0x{:x}", msg.header);
                }
            }
        }

        let Some(eval_res) = eval_res else {
            return Err(Error::Message("failed to evaluate pkl module".into()));
        };

        let Some(res) = eval_res.response.as_map() else {
            return Err(Error::PklMalformedResponse {
                message: "expected map in evaluate response".into(),
            });
        };
        let Some((_, result)) = res.iter().find(|(k, _v)| k.as_str() == Some("result")) else {
            // pkl module evaluation failed, return the error message from pkl
            if let Some((_, error)) = res.iter().find(|(k, _v)| k.as_str() == Some("error")) {
                return Err(Error::PklServerError {
                    pkl_error: error
                        .as_str()
                        .context("error message from pkl should be valid utf8")?
                        .to_owned(),
                });
            }

            return Err(Error::PklMalformedResponse {
                message: "expected result or error in evaluate response".into(),
            });
        };

        let slice = result
            .as_slice()
            .context("expected result to be a slice, got: {result:?}")?;
        let rmpv_ast: rmpv::Value = rmpv::decode::value::read_value(&mut &slice[..])?;

        _debug!("rmpv pkl module: {:#?}", rmpv_ast);
        let pkl_mod = decode_module(&rmpv_ast)?;

        Ok(pkl_mod)
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        let child_stdin = &mut self.stdin;

        let msg = CloseEvaluator {
            evaluator_id: self.evaluator_id,
        }
        .encode_msg()
        .expect("failed to encode close evaluator message");

        pkl_send_msg_one_way(child_stdin, &msg).expect("failed to close evaluator");
    }
}

fn start_pkl(pkl_debug: bool) -> Result<Child> {
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

fn pkl_send_msg_one_way(
    child_stdin: &mut std::process::ChildStdin,

    serialized_request: &[u8],
) -> Result<()> {
    child_stdin.write_all(serialized_request)?;
    child_stdin.flush()?;
    Ok(())
}

fn decode_pkl_message(value: &rmpv::Value) -> Option<PklServerMessage> {
    let decoded_array = value.as_array()?;
    let header = decoded_array.first()?.as_u64()?;
    let response = decoded_array.get(1)?.to_owned();
    Some(PklServerMessage { header, response })
}

fn pkl_send_msg_child(
    child_stdin: &mut std::process::ChildStdin,
    child_stdout: &mut std::process::ChildStdout,
    serialized_request: &[u8],
) -> Result<PklServerMessage> {
    child_stdin.write_all(serialized_request)?;
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

            Ok(PklServerMessage {
                header: message_header_hex,
                response: second.to_owned(),
            })
        }
        Err(e) => Err(Error::Message(format!(
            "\nfailed to decode value from pkl process: {e:?}",
        ))),
    }
}

pub(crate) fn recv_msg<R: Read>(reader: &mut R) -> std::result::Result<PklServerMessage, Error> {
    let data = rmpv::decode::read_value(reader)?;
    let pkl_msg = decode_pkl_message(&data).ok_or(Error::PklMalformedResponse {
        message: "Failed to decode pkl message format".to_string(),
    })?;
    Ok(pkl_msg)
}
