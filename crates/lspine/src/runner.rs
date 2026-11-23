use log::{error, info, warn};
use std::io::{BufRead, Write};

use crate::{
    LanguageServer,
    error::LSPError,
    model::{Envelope, InitializeRequestParams, Message, RequestId},
    transport::{read_message, write_message},
};

// JSON-RPC / LSP error codes
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;
const SERVER_NOT_INITIALIZED: i64 = -32002;

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Uninitialized,
    Running,
    ShuttingDown,
}

enum Flow {
    Continue,
    Exit(i32),
}

struct RpcError {
    code: i64,
    message: String,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<LSPError> for RpcError {
    fn from(e: LSPError) -> Self {
        Self::new(INTERNAL_ERROR, e.to_string())
    }
}

pub struct Runner<T: LanguageServer, R: BufRead, W: Write> {
    language_server: T,
    state: State,
    writer: W,
    reader: R,
}

impl<T: LanguageServer, R: BufRead, W: Write> Runner<T, R, W> {
    pub fn new(language_server: T, reader: R, writer: W) -> Self {
        Self {
            language_server,
            state: State::Uninitialized,
            writer,
            reader,
        }
    }

    pub fn run(&mut self) -> Result<i32, LSPError> {
        loop {
            if let Flow::Exit(code) = self.step()? {
                return Ok(code);
            }
        }
    }

    fn step(&mut self) -> Result<Flow, LSPError> {
        let Some(raw) = read_message(&mut self.reader)? else {
            warn!("client closed the stream without `exit`");
            return Ok(Flow::Exit(1));
        };

        let message = serde_json::from_str::<Envelope>(&raw)
            .map_err(LSPError::from)
            .and_then(Envelope::classify);
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                error!("unparseable message: {e}");
                self.respond_error(
                    serde_json::Value::Null,
                    RpcError::new(PARSE_ERROR, e.to_string()),
                )?;
                return Ok(Flow::Continue);
            }
        };

        match message {
            Message::Request { id, method, params } => {
                info!("request {method}");
                match self.handle_request(&method, params) {
                    Ok(result) => self.respond(id, result)?,
                    Err(e) => self.respond_error(id, e)?,
                }
                Ok(Flow::Continue)
            }
            Message::Notification { method, params } => {
                info!("notification {method}");
                if method == "exit" {
                    let code = if self.state == State::ShuttingDown {
                        0
                    } else {
                        1
                    };
                    return Ok(Flow::Exit(code));
                }
                if self.state == State::Running {
                    self.language_server.notification(&method, params);
                }
                Ok(Flow::Continue)
            }
            Message::Response { id } => {
                info!("ignoring response to {id}");
                Ok(Flow::Continue)
            }
        }
    }

    fn handle_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, RpcError> {
        match (self.state, method) {
            (State::Uninitialized, "initialize") => {
                let params: InitializeRequestParams = serde_json::from_value(params)
                    .map_err(|e| RpcError::new(INVALID_PARAMS, e.to_string()))?;
                let result = self.language_server.initialize(&params)?;
                self.state = State::Running;
                serde_json::to_value(result).map_err(|e| LSPError::from(e).into())
            }
            (State::Uninitialized, _) => Err(RpcError::new(
                SERVER_NOT_INITIALIZED,
                "server not initialized",
            )),
            (State::ShuttingDown, _) => {
                Err(RpcError::new(INVALID_REQUEST, "server is shutting down"))
            }
            (State::Running, "initialize") => {
                Err(RpcError::new(INVALID_REQUEST, "already initialized"))
            }
            (State::Running, "shutdown") => {
                self.language_server.shutdown()?;
                self.state = State::ShuttingDown;
                Ok(serde_json::Value::Null)
            }
            (State::Running, other) => Err(RpcError::new(
                METHOD_NOT_FOUND,
                format!("method not found: {other}"),
            )),
        }
    }

    fn respond(&mut self, id: RequestId, result: serde_json::Value) -> Result<(), LSPError> {
        write_message(
            &mut self.writer,
            &serde_json::json!({"jsonrpc": "2.0", "id": id, "result": result}),
        )
    }

    fn respond_error(&mut self, id: RequestId, e: RpcError) -> Result<(), LSPError> {
        write_message(
            &mut self.writer,
            &serde_json::json!({"jsonrpc": "2.0", "id": id, "error": {"code": e.code, "message": e.message}}),
        )
    }
}
