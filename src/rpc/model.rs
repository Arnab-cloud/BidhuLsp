#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Message {
    MRequest(Request),
    MResponse,
    MNotification(serde_json::Value),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Request {
    Initialize(IntializeRequest),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Response {
    Initialize(IntializeResult),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct IntializeRequest {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    pub params: IntializeRequestParams,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct IntializeRequestParams {
    #[serde(rename = "processId")]
    pub process_id: Option<i32>,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializeResponse {
    pub jsonrpc: String,
    pub id: i32,
    pub result: IntializeResult,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct IntializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
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
