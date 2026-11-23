use crate::error::LSPError;

pub struct WillSaveTextDocumentParams;
pub struct TextEdit;

pub type RequestId = serde_json::Value;

#[derive(serde::Deserialize, Debug)]
pub struct Envelope {
    pub id: Option<RequestId>,
    pub method: Option<String>,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Message {
    Request {
        id: RequestId,
        method: String,
        params: serde_json::Value,
    },
    Response {
        id: RequestId,
    },
    Notification {
        method: String,
        params: serde_json::Value,
    },
}

impl Envelope {
    pub fn classify(self) -> Result<Message, LSPError> {
        match (self.id, self.method) {
            (Some(id), Some(method)) => Ok(Message::Request {
                id,
                method,
                params: self.params,
            }),
            (Some(id), None) => Ok(Message::Response { id }),
            (None, Some(method)) => Ok(Message::Notification {
                method,
                params: self.params,
            }),
            (None, None) => Err(LSPError::Protocol(
                "message has neither id nor method".into(),
            )),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializeRequest {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    pub params: InitializeRequestParams,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializeRequestParams {
    #[serde(rename = "processId")]
    pub process_id: Option<i32>,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializeResponse {
    pub jsonrpc: String,
    pub id: i32,
    pub result: InitializeResult,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ServerCapabilities {
    #[serde(rename = "hoverProvider")]
    pub hover_provider: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}
