use crate::error::LSPError;

pub type RequestId = serde_json::Value;
pub type DocumentUri = String;

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
pub struct InitializeRequestParams {
    #[serde(rename = "processId")]
    pub process_id: Option<i32>,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct WillSaveWaitUntilResult(pub Option<Box<[TextEdit]>>);

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

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct InitializedParams;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct DidOpenTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentItem,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct DidChangeTextDocumentParams {
    /// The document that did change. The version number points
    /// to the version after all provided content changes have
    /// been applied.
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,

    /// The actual content changes. The content changes describe single state
    /// changes to the document. So if there are two content changes c1 (at
    /// array index 0) and c2 (at array index 1) for a document in state S then
    /// c1 moves the document from S to S' and c2 from S' to S''. So c1 is
    /// computed on the state S and c2 is computed on the state S'.
    ///
    /// To mirror the content of a document using change events use the following
    /// approach:
    /// - start with the same initial content
    /// - apply the 'textDocument/didChange' notifications in the order you
    ///   receive them.
    /// - apply the `TextDocumentContentChangeEvent`s in a single notification
    ///   in the order you receive them.
    #[serde(rename = "contentChanges")]
    content_changes: Box<[TextDocumentContentChangeEvent]>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct DidCloseTextDocumentParams {
    /// The document that was closed.
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,
}

/// The parameters send in a will save text document notification.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct WillSaveTextDocumentParams {
    /// The document that will be saved.
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,

    /// The 'TextDocumentSaveReason'.
    reason: TextDocumentSaveReason,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct DidSaveTextDocumentParams {
    /// The document that was saved.
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,

    /// Optional the content when saved. Depends on the includeText value
    /// when the save notification was requested.
    text: Option<String>,
}

/// The parameters sent in notifications/requests for user-initiated renames
/// of files.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct RenameFilesParams {
    /// An array of all files/folders renamed in this operation. When a folder
    /// is renamed, only the folder will be included, and not its children.
    files: Box<[FileRename]>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct TextDocumentItem {
    /// The text document's URI.
    pub uri: DocumentUri,

    /// The text document's language identifier.
    #[serde(rename = "languageId")]
    pub language_id: String,

    /// The version number of this document (it will increase after each
    /// change, including undo/redo).
    pub version: i32,

    /// The content of the opened text document.
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct TextDocumentIdentifier {
    /// The text document's URI.
    pub uri: DocumentUri,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct VersionedTextDocumentIdentifier {
    /// The text document's URI.
    pub uri: DocumentUri,

    /// The version number of this document.
    ///
    /// The version number of a document will increase after each change,
    /// including undo/redo. The number doesn't need to be consecutive.
    pub version: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
/// An event describing a change to a text document. If only a text is provided
/// it is considered to be the full content of the document.
pub enum TextDocumentContentChangeEvent {
    TextDocumentContentChangePartial {
        /// The range of the document that changed.
        range: Range,

        /// The optional length of the range that got replaced.
        ///
        /// @deprecated use range instead.
        #[serde(rename = "rangeLength")]
        range_length: Option<u32>,

        /// The new text for the provided range.
        text: String,
    },
    TextDocumentContentChangeWholeDocument {
        /// The new text of the whole document.
        text: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Range {
    /// The range's start position.
    pub start: Position,

    /// The range's end position.
    pub end: Position,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: u32,

    /// Character offset on a line in a document (zero-based). The meaning of this
    /// offset is determined by the negotiated `PositionEncodingKind`.
    ///
    /// If the character value is greater than the line length it defaults back
    /// to the line length.
    pub character: u32,
}

/// Represents reasons why a text document is sav
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum TextDocumentSaveReason {
    /// Manually triggered, e.g. by the user pressing save, by starting
    /// debugging, or by an API call.
    Manual = 1,

    /// Automatic after a delay.
    AfterDelay = 2,

    /// When the editor lost focus.
    FocusOut = 3,
}

/// A textual edit applicable to a text document.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct TextEdit {
    /// The range of the text document to be manipulated. To insert
    /// text into a document, create a range where start === end.
    range: Range,

    /// The string to be inserted. For delete operations, use an
    /// empty string.
    #[serde(rename = "newText")]
    new_text: String,
}

/// Represents information on a file/folder rename.
///
/// @since 3.16.0
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct FileRename {
    /// A file:// URI for the original location of the file/folder being renamed.
    #[serde(rename = "oldUri")]
    old_uri: String,

    /// A file:// URI for the new location of the file/folder being renamed.
    #[serde(rename = "newUri")]
    new_uri: String,
}
