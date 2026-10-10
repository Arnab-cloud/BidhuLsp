use std::io::BufRead;

pub mod model;

pub fn read_message<T: BufRead>(reader: &mut T) -> Result<String, ()> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap(); // NOTE: assuming the header has only `Content-Type`
    reader.read_line(&mut buffer).unwrap();
    let content_len: u32 = buffer["Content-Length: ".len()..].trim().parse().unwrap();
    buffer.clear();
    let mut line = vec![0u8; content_len as usize];
    reader.read_exact(&mut line).unwrap();
    Ok(String::from_utf8(line).unwrap())
}

pub fn create_rpc_message<T: serde::Serialize>(content: T) -> Result<String, ()> {
    let content_str = serde_json::to_string(&content).unwrap();
    let header = format!("Content-Length: {}\r\n", content_str.len());
    Ok(format!("{header}\r\n{content_str}"))
}

pub fn parse_message(content: &str) -> model::Message {
    let value: serde_json::Value = serde_json::from_str(content).unwrap();
    match value.get("id") {
        Some(_) => {
            let requst: model::IntializeRequest = serde_json::from_value(value).unwrap();
            return model::Message::MRequest(model::Request::Initialize(requst));
        }
        _ => model::Message::MNotification(value),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(serde::Serialize)]
    struct SampleRequest {
        jsonrpc: String,
        id: u32,
        method: String,
    }

    #[test]
    fn test_read_rpc_message() {
        let content = "Content-Length: 59\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"textDocument/completion\"}}some extrabytes";
        let msg = read_message(&mut content.as_bytes()).unwrap();
        assert_eq!(
            msg,
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"textDocument/completion\"}"
        )
    }

    #[test]
    fn test_creat_rpc_message() {
        let content = SampleRequest {
            jsonrpc: String::from("2.0"),
            id: 1,
            method: String::from("textDocument/completion"),
        };
        let content_str = "Content-Length: 59\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"textDocument/completion\"}";
        assert_eq!(content_str, create_rpc_message(content).unwrap())
    }

    #[test]
    fn test_creat_rpc_message_for_response() {
        let response = model::InitializeResponse {
            jsonrpc: String::from("2.0"),
            id: 1,
            result: model::IntializeResult {
                capabilities: model::ServerCapabilities {
                    hover_provider: false,
                },
                server_info: model::ServerInfo {
                    name: String::from("bidhulsp"),
                    version: String::from("0.0.0.0.0.1"),
                },
            },
        };
        let msg_body = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"capabilities\":{\"hoverProvider\":false},\"serverInfo\":{\"name\":\"bidhulsp\",\"version\":\"0.0.0.0.0.1\"}}}";
        let msg_str = format!("Content-Length: {}\r\n\r\n{}", msg_body.len(), msg_body);
        assert_eq!(msg_str, create_rpc_message(response).unwrap())
    }

    #[test]
    fn test_parse_message() {
        let content_str = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\", \"params\": {\"processId\": 1, \"clientInfo\": {\"name\": \"nvim\", \"version\": \"0.1\"}}}";
        let content =
            model::Message::MRequest(model::Request::Initialize(model::IntializeRequest {
                jsonrpc: String::from("2.0"),
                id: 1,
                method: String::from("initialize"),
                params: model::IntializeRequestParams {
                    process_id: Some(1),
                    client_info: model::ClientInfo {
                        name: String::from("nvim"),
                        version: String::from("0.1"),
                    },
                },
            }));
        assert_eq!(parse_message(content_str), content)
    }
}
