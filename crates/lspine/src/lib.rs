mod error;
mod model;
mod runner;
mod transport;

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

    fn notification(&mut self, _method: &str, _params: serde_json::Value) {}

    // TODO: client/registerCapability
    // TODO: client/unregisterCapability

    fn will_save_wait_until_text_document(
        &mut self,
        _params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>, LSPError> {
        Ok(None)
    }
}
