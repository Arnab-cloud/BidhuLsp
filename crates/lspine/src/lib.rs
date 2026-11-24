mod error;
pub mod model;
mod runner;
pub mod transport;

pub use error::*;
pub use model::*;
pub use runner::*;
pub use transport::*;

// for now the error is io::Error, TODO: add
// custom error
pub trait LanguageServer {
    fn initialize(
        &mut self,
        params: &InitializeRequestParams,
    ) -> Result<InitializeResult, LSPError>;

    fn shutdown(&mut self) -> Result<(), LSPError>;

    fn notification(&mut self, _method: &str) {}

    // TODO: client/registerCapability
    // TODO: client/unregisterCapability

    fn did_open_text_docuement(&mut self, _params: DidOpenTextDocumentParams) {
        self.notification("textDocument/didOpen");
    }

    fn did_change_text_docuement(&mut self, _params: DidChangeTextDocumentParams) {
        self.notification("textDocument/didChange");
    }

    fn did_close_text_docuement(&mut self, _params: DidCloseTextDocumentParams) {
        self.notification("textDocument/didClose");
    }

    fn will_save_text_docuement(&mut self, _params: WillSaveTextDocumentParams) {
        self.notification("textDocument/willSave");
    }

    fn will_save_wait_until_text_document(
        &mut self,
        _params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>, LSPError> {
        Ok(None)
    }
}
