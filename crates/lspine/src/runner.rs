use log::{error, info, warn};
use std::io::{BufRead, Write};

use crate::{
    LanguageServer,
    error::LSPError,
    model::{
        CallHierarchyPrepareParams, DeclarationParams, DefinitionParams,
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, Envelope, ImplementationParams, InitializeRequestParams,
        Message, ReferenceParams, RenameFilesParams, RequestId, TypeDefinitionParams,
        WillSaveTextDocumentParams,
    },
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

    pub fn into_parts(self) -> (T, W) {
        (self.language_server, self.writer)
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
                if let Ok(Some(exit_code)) = self.handle_notification(&method, params) {
                    // ignoring the error
                    return Ok(Flow::Exit(exit_code));
                }
                Ok(Flow::Continue)
            }
            Message::Response { id } => {
                info!("ignoring response to {id}");
                Ok(Flow::Continue)
            }
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Option<i32>, RpcError> {
        match (self.state, method) {
            (State::Uninitialized, _) => Err(RpcError::new(
                SERVER_NOT_INITIALIZED,
                "server not initialized",
            )),
            (State::Running, "textDocument/didOpen") => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                self.language_server.did_open_text_docuement(params);
                Ok(None)
            }
            (State::Running, "textDocument/didChange") => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                self.language_server.did_change_text_docuement(params);
                Ok(None)
            }
            (State::Running, "textDocument/didClose") => {
                let params: DidCloseTextDocumentParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                self.language_server.did_close_text_docuement(params);
                Ok(None)
            }
            (State::Running, "textDocument/willSave") => {
                let params: WillSaveTextDocumentParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                self.language_server.will_save_text_docuement(params);
                Ok(None)
            }
            (State::Running, "textDocument/didSave") => {
                let params: DidSaveTextDocumentParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                self.language_server.did_save_text_docuement(params);
                Ok(None)
            }
            (State::Running, "workspace/didRenameFiles") => {
                let params: RenameFilesParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                self.language_server.did_rename_files_workspace(params);
                Ok(None)
            }
            (State::Running, "exit") => Ok(Some(1)),
            (State::Running, _) => {
                self.language_server.notification(method);
                Ok(None)
            }
            (State::ShuttingDown, "exit") => Ok(Some(0)),
            (State::ShuttingDown, _) => {
                Err(RpcError::new(INVALID_REQUEST, "server is shutting down"))
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
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.initialize(&params)?;
                self.state = State::Running;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
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
            (State::Running, "textDocument/willSaveWaitUntil") => {
                let params: WillSaveTextDocumentParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let text_edits = self
                    .language_server
                    .will_save_wait_until_text_document(&params)?;
                serde_json::to_value(text_edits).map_err(|err| LSPError::from(err).into())
            }
            (State::Running, "textDocument/declaration") => {
                let params: DeclarationParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.go_to_declaration(params)?;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
            }

            (State::Running, "textDocument/definition") => {
                let params: DefinitionParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.go_to_definition(params)?;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
            }
            (State::Running, "textDocument/typeDefinition") => {
                let params: TypeDefinitionParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.go_to_type_definition(params)?;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
            }
            (State::Running, "textDocument/implementation") => {
                let params: ImplementationParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.go_to_implementation(params)?;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
            }
            (State::Running, "textDocument/references") => {
                let params: ReferenceParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.find_references(params)?;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
            }
            (State::Running, "textDocument/prepareCallHierarchy") => {
                let params: CallHierarchyPrepareParams = serde_json::from_value(params)
                    .map_err(|err| RpcError::new(INVALID_PARAMS, err.to_string()))?;
                let result = self.language_server.prepare_call_hierarchy(params)?;
                serde_json::to_value(result).map_err(|err| LSPError::from(err).into())
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
