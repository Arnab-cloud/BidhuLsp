pub mod model;

use std::{
    fmt,
    io::{self, Write},
};

use log::{error, info};

use model::{
    InitializeRequestParams, InitializeResponse, InitializeResult, Message, TextEdit,
    WillSaveTextDocumentParams,
};

use crate::languageserver::model::{Request, ServerCapabilities, ServerInfo};

#[derive(Debug)]
pub enum LSPError {
    Other(String),
}

impl fmt::Display for LSPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

/// Reads a complete RPC message from `reader`
/// > [WARNING]: assuming the first line in the message header has `Content-Type`
/// and the second line has `Content-Length`
/// > [WARNING]: for any error the function panics
fn read_message<B: io::BufRead>(reader: &mut B) -> Result<String, ()> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap(); // NOTE: assuming the header has only `Content-Type`
    reader.read_line(&mut buffer).unwrap();
    let content_len: u32 = buffer["Content-Length: ".len()..].trim().parse().unwrap();
    buffer.clear();
    let mut line = vec![0u8; content_len as usize];
    reader.read_exact(&mut line).unwrap();
    Ok(String::from_utf8(line).unwrap())
}

/// Tries to DeSerialize the `content`
/// > [WARNING]: for any error the function panics
pub fn parse_message(content: &str) -> Message {
    let value: serde_json::Value = serde_json::from_str(content).unwrap();
    match value.get("id") {
        Some(_) => {
            let requst: model::IntializeRequest = serde_json::from_value(value).unwrap();
            return model::Message::MRequest(model::Request::Initialize(requst));
        }
        _ => model::Message::MNotification(value),
    }
}

/// tries to Serialize a Serializeable `content` type
/// > [WARNING]: for any error the function panics
pub fn create_rpc_message<M: serde::Serialize>(content: M) -> Result<String, ()> {
    let content_str = serde_json::to_string(&content).unwrap();
    let header = format!("Content-Length: {}\r\n", content_str.len());
    Ok(format!("{header}\r\n{content_str}"))
}

// for now the error is io::Error, TODO: add
// custom error
pub trait LanguageServer {
    fn initialize(
        &mut self,
        params: &InitializeRequestParams,
    ) -> Result<InitializeResult, io::Error>;

    fn shutdown(&mut self) -> Result<(), io::Error>;

    // TODO: client/registerCapability
    // TODO: client/unregisterCapability

    fn will_save_wait_until_text_document(
        &mut self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>, io::Error>;
}

struct Runner<T: LanguageServer> {
    language_server: T,
}

impl<T: LanguageServer> Runner<T> {
    pub fn new(language_server: T) -> Self {
        Self { language_server }
    }

    fn run(&mut self) {
        let mut stdin = io::BufReader::new(io::stdin());
        loop {
            match read_message(&mut stdin) {
                Ok(msg) => {
                    let message = &parse_message(&msg);
                    match message {
                        Message::MRequest(request) => match request {
                            Request::Initialize(intialize_request) => {
                                if let Ok(result) =
                                    self.language_server.initialize(&intialize_request.params)
                                {
                                    let response = InitializeResponse {
                                        jsonrpc: String::from("2.0"),
                                        id: 1,
                                        result,
                                    };
                                    let msg = create_rpc_message(response).unwrap();
                                    print!("{}", msg);
                                    io::stdout().flush().unwrap();
                                } else {
                                    todo!();
                                }
                            }
                        },
                        Message::MNotification(val) => info!("{:?}", val.get("method")),
                        Message::MResponse => todo!(),
                    }
                }
                Err(_) => error!("ERROR: Failed to parse the message"),
            }
        }
    }
}
