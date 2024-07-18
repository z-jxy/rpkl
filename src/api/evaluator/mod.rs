use std::{
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::{
    error::{Error, Result},
    utils,
};

pub const EVALUATE_RESPONSE: u64 = 0x24;

use outgoing::{
    pack_msg, ClientModuleReader, CloseEvaluator, CreateEvaluator, EvaluateRequest, OutgoingMessage,
};
use responses::PklServerResponseRaw;

use crate::{api::decoder::pkl_eval_module, pkl::PklMod};

pub mod outgoing;
pub mod responses;

#[cfg(feature = "trace")]
use tracing::debug;

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
        let mut child = start_pkl(false).map_err(|_e| Error::PklProcessStart)?;
        let child_stdin = child.stdin.as_mut().unwrap();
        let mut child_stdout = child.stdout.take().unwrap();

        let request = OutgoingMessage::CreateEvaluator(CreateEvaluator {
            request_id: 135,
            allowed_modules: vec![
                "pkl:".to_string(),
                "repl:".to_string(),
                "file:".to_string(),
                "customfs:".to_string(),
            ],
            client_module_readers: vec![ClientModuleReader {
                scheme: "customfs".to_string(),
                has_hierarchical_uris: true,
                is_globbable: true,
                is_local: true,
            }],
        });

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
            request_id: 9805131,
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
