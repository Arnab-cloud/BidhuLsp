use std::io;

use crate::error::LSPError;

/// Reads a complete RPC message from `reader`
pub fn read_message<B: io::BufRead>(reader: &mut B) -> Result<Option<String>, LSPError> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return if content_length.is_none() {
                Ok(None)
            } else {
                Err(LSPError::Protocol("EOF inside message header".into()))
            };
        }
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        let Some((name, value)) = header.split_once(':') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case("content-length") {
            continue;
        }
        content_length = Some(value.trim().parse().map_err(|err| {
            LSPError::Protocol(format!("bad Content-Length: {value}\nError: {err}"))
        })?)
    }
    let content_length =
        content_length.ok_or_else(|| LSPError::Protocol("missing Content-Length".into()))?;
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|err| LSPError::Protocol(format!("body is not valid utf-8: {err}")))
}

pub fn write_message<W: io::Write, T: serde::Serialize>(
    writer: &mut W,
    content: &T,
) -> Result<(), LSPError> {
    writer.write_all(&encode_message(content)?.as_bytes())?;
    writer.flush()?;
    Ok(())
}

/// Serializes a Serializeable `content` type and Prepends a header
pub fn encode_message<T: serde::Serialize>(content: &T) -> Result<String, LSPError> {
    let body = serde_json::to_string(&content)?;
    Ok(format!("Content-Length: {}\r\n\r\n{}", body.len(), body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const BODY: &str = r#"{"jsonrpc":"2.0","id":1,"method":"x"}"#;

    fn framed(body: &str) -> Vec<u8> {
        format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
    }

    #[test]
    fn reads_one_message_and_leaves_the_rest() {
        let mut data = framed(BODY);
        data.extend_from_slice(b"trailing");
        let mut r = &data[..];
        assert_eq!(read_message(&mut r).unwrap().as_deref(), Some(BODY));
        assert_eq!(r, b"trailing");
    }

    #[test]
    fn reads_back_to_back_messages_then_eof() {
        let mut data = framed(BODY);
        data.extend(framed(BODY));
        let mut r = &data[..];
        assert!(read_message(&mut r).unwrap().is_some());
        assert!(read_message(&mut r).unwrap().is_some());
        assert!(read_message(&mut r).unwrap().is_none());
    }

    #[test]
    fn ignores_content_type_and_header_case() {
        let data = format!(
            "content-type: application/vscode-jsonrpc; charset=utf-8\r\ncontent-length: {}\r\n\r\n{}",
            BODY.len(),
            BODY
        );
        assert_eq!(
            read_message(&mut data.as_bytes()).unwrap().as_deref(),
            Some(BODY)
        );
    }

    #[test]
    fn length_is_in_bytes_not_chars() {
        let body = r#"{"s":"héllo"}"#; // 'é' is 2 bytes
        assert_eq!(
            read_message(&mut &framed(body)[..]).unwrap().as_deref(),
            Some(body)
        );
    }

    #[test]
    fn errors_on_missing_length_and_truncated_body() {
        assert!(read_message(&mut &b"X: 1\r\n\r\n{}"[..]).is_err());
        assert!(read_message(&mut &b"Content-Length: 50\r\n\r\n{}"[..]).is_err());
    }

    #[test]
    fn encodes_with_correct_length() {
        let msg = encode_message(&json!({"jsonrpc":"2.0","id":1})).unwrap();
        let (header, body) = msg.split_once("\r\n\r\n").unwrap();
        assert_eq!(header, format!("Content-Length: {}", body.len()));
    }
}
