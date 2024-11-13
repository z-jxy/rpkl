use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::{
    error::{Error, Result},
    utils::{self, macros::_trace},
};

use outgoing::{pack_msg, CloseEvaluator, CreateEvaluator, EvaluateRequest, OutgoingMessage};
use responses::PklServerResponseRaw;

use crate::{api::decoder::pkl_eval_module, pkl::PklMod};

pub mod outgoing;
pub mod responses;

#[cfg(feature = "trace")]
use tracing::debug;

pub(crate) const EVALUATE_RESPONSE: u64 = 0x24;
pub(crate) const CREATE_EVALUATOR_REQUEST_ID: u64 = 135;
pub(crate) const OUTGOING_MESSAGE_REQUEST_ID: u64 = 9805131;

// options that can be provided to the evaluator, such as properties (-p flag from CLI)
pub struct EvaluatorOptions {
    /// Properties to pass to the evaluator. Used to read from `props:` in `.pkl` files.
    pub properties: Option<HashMap<String, String>>,
}

impl Default for EvaluatorOptions {
    fn default() -> Self {
        Self {
            properties: Some(HashMap::new()),
        }
    }
}

impl EvaluatorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    // Add a property to the evaluator options map
    pub fn property(mut self, key: String, value: String) -> Self {
        if let Some(properties) = self.properties.as_mut() {
            properties.insert(key, value);
        } else {
            let mut map = HashMap::new();
            map.insert(key, value);
            self.properties = Some(map);
        }
        self
    }

    // Set properties for the evaluator. This will replace any existing properties
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
}

pub struct Evaluator {
    pub evaluator_id: i64,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
}

impl Evaluator {
    pub fn id(&self) -> i64 {
        self.evaluator_id
    }

    pub fn new() -> Result<Self> {
        return Self::new_from_options(None);
    }

    pub fn new_from_options(options: Option<EvaluatorOptions>) -> Result<Self> {
        let mut child = start_pkl(false).map_err(|_e| Error::PklProcessStart)?;
        let child_stdin = child.stdin.as_mut().unwrap();
        let mut child_stdout = child.stdout.take().unwrap();

        let mut evaluator_message = CreateEvaluator::default();
        if let Some(options) = options {
            if let Some(props) = options.properties {
                evaluator_message.properties = Some(props);
            }
        }

        let request = OutgoingMessage::CreateEvaluator(evaluator_message);

        let serialized_request = pack_msg(request);

        let create_eval_response =
            pkl_send_msg_raw(child_stdin, &mut child_stdout, serialized_request)
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
        })
    }

    pub fn evaluate_module(&mut self, path: PathBuf) -> Result<PklMod> {
        let evaluator_id = self.id();
        let mut child_stdin = &mut self.stdin;
        let mut child_stdout = &mut self.stdout;

        let path = utils::canonicalize(path)
            .map_err(|_e| Error::Message("failed to canonicalize pkl module path".into()))?;

        let msg = OutgoingMessage::EvaluateRequest(EvaluateRequest {
            request_id: OUTGOING_MESSAGE_REQUEST_ID,
            evaluator_id: evaluator_id,
            module_uri: format!("file://{}", path.to_str().unwrap()),
        });

        let serialized_eval_req = pack_msg(msg);
        // rmp_serde::encode::write(&mut serialized_eval_req, &eval_req).unwrap();
        let eval_res = pkl_send_msg_raw(&mut child_stdin, &mut child_stdout, serialized_eval_req)?;

        if eval_res.header != EVALUATE_RESPONSE {
            todo!("handle case when header is not 0x24");
            // return Err(anyhow::anyhow!("expected 0x24, got 0x{:X}", eval_res.header));
        }

        _trace!("eval_res header: {:?}", eval_res.header);

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

        #[cfg(feature = "trace")]
        debug!("rmpv pkl module: {:#?}", rmpv_ast);

        let pkl_mod = pkl_eval_module(&rmpv_ast)?;

        Ok(pkl_mod)
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        let mut child_stdin = &mut self.stdin;

        let msg = pack_msg(OutgoingMessage::CloseEvaluator(CloseEvaluator {
            evaluator_id: self.evaluator_id,
        }));

        let _ = pkl_send_msg_one_way(&mut child_stdin, msg).expect("failed to close evaluator");
    }
}

pub fn start_pkl(pkl_debug: bool) -> Result<Child> {
    let mut command = Command::new("pkl");

    command
        .arg("server") // Replace with actual command
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

pub fn pkl_send_msg_raw(
    child_stdin: &mut std::process::ChildStdin,
    child_stdout: &mut std::process::ChildStdout,
    serialized_request: Vec<u8>,
) -> Result<PklServerResponseRaw> {
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

            return Ok(PklServerResponseRaw {
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
