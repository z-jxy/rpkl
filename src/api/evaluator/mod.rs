use std::{
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

pub const EVALUATE_RESPONSE: u64 = 0x24;

use serde_json::{json, Value};

use crate::{api::parser::pkl_eval_module, pkl::PklMod};

use self::responses::{EvaluatorResponse, PklServerResponse, PklServerResponse2};

pub mod responses;
pub struct Evaluator {
    pub evaluator_id: i64,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
}
impl Evaluator {
    pub fn id(&self) -> i64 {
        self.evaluator_id
    }

    pub fn new() -> anyhow::Result<Self> {
        let mut child = start_pkl(false)?;
        let child_stdin = child.stdin.as_mut().unwrap();
        let mut child_stdout = child.stdout.take().unwrap();
        // Create a CreateEvaluatorRequest instance
        const CREATE_EVALUATOR_REQUEST_SIZE: usize = 140;
        let request = json!([
          0x20,
            {
                "requestId": 135,
                "allowedModules": ["pkl:", "repl:", "file:", "customfs:"],
                "clientModuleReaders": [
                  {
                    "scheme": "customfs",
                    "hasHierarchicalUris": true,
                    "isGlobbable": true,
                    "isLocal": true
                  }
                ]
            }
        ]);

        let mut serialized_request = Vec::with_capacity(CREATE_EVALUATOR_REQUEST_SIZE as usize);
        // Serialize the request to a binary format
        rmp_serde::encode::write(&mut serialized_request, &request).unwrap();

        let create_eval_response =
            pkl_send_msg::<EvaluatorResponse>(child_stdin, &mut child_stdout, serialized_request)?;

        Ok(Evaluator {
            evaluator_id: create_eval_response.response.evaluator_id,
            stdin: child.stdin.take().unwrap(),
            stdout: child_stdout,
        })
    }

    pub fn evaluate_module(&mut self, path: PathBuf) -> anyhow::Result<PklMod> {
        let evaluator_id = self.id();
        let mut child_stdin = &mut self.stdin;
        let mut child_stdout = &mut self.stdout;

        let path = path.canonicalize()?;

        let eval_req = json!([
          0x23,
          {
            "requestId": 9805131,
            "evaluatorId": evaluator_id,
            "moduleUri": format!("file://{}", path.to_str().unwrap()),
          }
        ]);

        let mut serialized_eval_req = Vec::new();
        rmp_serde::encode::write(&mut serialized_eval_req, &eval_req).unwrap();
        let eval_res =
            pkl_send_msg_v2::<Value>(&mut child_stdin, &mut child_stdout, serialized_eval_req)?;

        if eval_res.header != EVALUATE_RESPONSE {
            todo!("handle case when header is not 0x24");
            // return Err(anyhow::anyhow!("expected 0x24, got 0x{:X}", eval_res.header));
        }

        let res = eval_res.response.as_map().unwrap();

        let Some((_, result)) = res.iter().find(|(k, _v)| k.as_str() == Some("result")) else {
            return Err(anyhow::anyhow!("expected result in response"));
        };

        let slice = result.as_slice().unwrap();
        let rmpv_ast: rmpv::Value = rmpv::decode::value::read_value(&mut &slice[..])?;

        let pkl_mod = pkl_eval_module(&rmpv_ast)?;

        Ok(pkl_mod)
    }

    pub fn evaluate_module_as_slice(&mut self, path: PathBuf) -> anyhow::Result<Vec<u8>> {
        let evaluator_id = self.id();
        let mut child_stdin = &mut self.stdin;
        let mut child_stdout = &mut self.stdout;

        let path = path.canonicalize()?;

        let eval_req = json!([
          0x23,
          {
            "requestId": 9805131,
            "evaluatorId": evaluator_id,
            "moduleUri": format!("file://{}", path.to_str().unwrap()),
          }
        ]);

        let mut serialized_eval_req = Vec::new();
        rmp_serde::encode::write(&mut serialized_eval_req, &eval_req).unwrap();
        let eval_res =
            pkl_send_msg_v2::<Value>(&mut child_stdin, &mut child_stdout, serialized_eval_req)?;

        if eval_res.header != EVALUATE_RESPONSE {
            todo!("handle case when header is not 0x24");
            // return Err(anyhow::anyhow!("expected 0x24, got 0x{:X}", eval_res.header));
        }

        let res = eval_res.response.as_map().unwrap();

        let Some((_, result)) = res.iter().find(|(k, _v)| k.as_str() == Some("result")) else {
            return Err(anyhow::anyhow!("expected result in response"));
        };

        let slice = result.as_slice().unwrap();
        Ok(slice.to_vec())
    }

    pub fn close(self) -> anyhow::Result<()> {
        let eval_req = json!([
            0x22,
          {
            "evaluatorId": self.evaluator_id,
          }
        ]);

        let mut serialized_eval_req = Vec::new();
        rmp_serde::encode::write(&mut serialized_eval_req, &eval_req).unwrap();
        Ok(())
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        let mut child_stdin = &mut self.stdin;

        let eval_req = json!([
            0x22,
          {
            "evaluatorId": self.evaluator_id,
          }
        ]);

        let mut serialized_eval_req = Vec::new();
        rmp_serde::encode::write(&mut serialized_eval_req, &eval_req).unwrap();
        let _ = pkl_send_msg_one_way(&mut child_stdin, serialized_eval_req)
            .expect("failed to close evaluator");
    }
}

pub fn start_pkl(pkl_debug: bool) -> anyhow::Result<Child> {
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
) -> anyhow::Result<()> {
    child_stdin.write_all(&serialized_request)?;
    child_stdin.flush()?;
    Ok(())
}

pub fn pkl_send_msg<T>(
    child_stdin: &mut std::process::ChildStdin,
    child_stdout: &mut std::process::ChildStdout,
    serialized_request: Vec<u8>,
) -> anyhow::Result<PklServerResponse<T>>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
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
            let response: T = serde_json::from_str(&second.to_string()).expect(
                "failed to deserialize response from server, expected response to be a json object",
            );
            // let map_t = second.as_map().unwrap().to_owned();

            return Ok(PklServerResponse {
                header: message_header_hex,
                response: response,
            });
        }
        Err(e) => Err(anyhow::anyhow!("\n@[decoder]error: {:?}", e)),
    }
}

pub fn pkl_send_msg_v2<T>(
    child_stdin: &mut std::process::ChildStdin,
    child_stdout: &mut std::process::ChildStdout,
    serialized_request: Vec<u8>,
) -> anyhow::Result<PklServerResponse2> {
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

            return Ok(PklServerResponse2 {
                header: message_header_hex,
                response: second.to_owned(),
            });
        }
        Err(e) => Err(anyhow::anyhow!("\n@[decoder]error: {:?}", e)),
    }
}
