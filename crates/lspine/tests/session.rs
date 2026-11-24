//! Scripted client sessions against a MockServer, fully in memory.

use lspine::{
    LSPError, LanguageServer, Runner,
    model::{
        ClientInfo, InitializeRequestParams, InitializeResult, ServerCapabilities, ServerInfo,
    },
    transport::{encode_message, read_message},
};
use serde_json::{Value, json};

#[derive(Default)]
struct MockServer {
    initialized_with: Option<InitializeRequestParams>,
    notifications: Vec<String>,
    shutdown_called: bool,
}

impl LanguageServer for MockServer {
    fn initialize(&mut self, p: &InitializeRequestParams) -> Result<InitializeResult, LSPError> {
        self.initialized_with = Some(InitializeRequestParams {
            process_id: p.process_id,
            client_info: ClientInfo {
                name: "test".into(),
                version: "1".into(),
            },
        });
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: false,
            },
            server_info: ServerInfo {
                name: "mock".into(),
                version: "0".into(),
            },
        })
    }
    fn shutdown(&mut self) -> Result<(), LSPError> {
        self.shutdown_called = true;
        Ok(())
    }
    fn notification(&mut self, method: &str) {
        self.notifications.push(method.to_string());
    }
}

fn req(id: i64, method: &str, params: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
}
fn notif(method: &str) -> Value {
    json!({"jsonrpc":"2.0","method":method,"params":{}})
}
fn init() -> Value {
    req(
        1,
        "initialize",
        json!({"processId": 42, "clientInfo": {"name": "test"}}),
    )
}

/// Plays `script` into a fresh runner; returns (exit code, server, replies).
fn play(script: &[Value]) -> (i32, MockServer, Vec<Value>) {
    let input: String = script.iter().map(|m| encode_message(m).unwrap()).collect();
    let mut runner = Runner::new(MockServer::default(), input.as_bytes(), Vec::new());
    let code = runner.run().unwrap();
    let (server, out) = runner.into_parts();

    let mut replies = Vec::new();
    let mut r = &out[..];
    while let Some(raw) = read_message(&mut r).unwrap() {
        replies.push(serde_json::from_str(&raw).unwrap());
    }
    (code, server, replies)
}

#[test]
fn happy_path_exits_zero() {
    let (code, server, replies) = play(&[
        init(),
        notif("initialized"),
        notif("textDocument/didOpen"),
        req(2, "shutdown", Value::Null),
        notif("exit"),
    ]);
    assert_eq!(code, 0);
    assert!(server.shutdown_called);
    assert_eq!(server.initialized_with.unwrap().process_id, Some(42));
    assert_eq!(
        server.notifications,
        ["initialized", "textDocument/didOpen"]
    );

    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["id"], 1);
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "mock");
    assert_eq!(replies[1]["id"], 2);
    assert!(replies[1]["result"].is_null());
}

#[test]
fn response_echoes_the_request_id_including_strings() {
    let (_, _, replies) = play(&[json!({
        "jsonrpc":"2.0","id":"abc","method":"initialize","params":{"processId":null}
    })]);
    assert_eq!(replies[0]["id"], "abc");
}

#[test]
fn exit_without_shutdown_exits_one() {
    let (code, _, _) = play(&[init(), notif("exit")]);
    assert_eq!(code, 1);
}

#[test]
fn eof_without_exit_exits_one() {
    let (code, _, _) = play(&[init()]);
    assert_eq!(code, 1);
}

#[test]
fn request_before_initialize_is_rejected() {
    let (_, _, replies) = play(&[req(1, "textDocument/hover", json!({}))]);
    assert_eq!(replies[0]["error"]["code"], -32002);
}

#[test]
fn unknown_method_gets_method_not_found() {
    let (_, _, replies) = play(&[init(), req(2, "bogus/method", json!({}))]);
    assert_eq!(replies[1]["error"]["code"], -32601);
}

#[test]
fn requests_after_shutdown_are_rejected() {
    let (_, _, replies) = play(&[
        init(),
        req(2, "shutdown", Value::Null),
        req(3, "textDocument/hover", json!({})),
    ]);
    assert_eq!(replies[2]["error"]["code"], -32600);
}

#[test]
fn notifications_before_initialize_are_dropped() {
    let (_, server, _) = play(&[notif("textDocument/didOpen"), init()]);
    assert!(server.notifications.is_empty());
}

#[test]
fn bad_initialize_params_give_invalid_params() {
    let (_, _, replies) = play(&[req(1, "initialize", json!({"processId": "not-a-number"}))]);
    assert_eq!(replies[0]["error"]["code"], -32602);
}
