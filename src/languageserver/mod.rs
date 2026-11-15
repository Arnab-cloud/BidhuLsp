pub mod model;

use std::io::Error;

use model::{InitializeParams, InitializeResult, TextEdit, WillSaveTextDocumentParams};

// for now the error is io::Error, TODO: add
// custom error
pub trait LanguageServer {
    fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult, Error>;

    fn shutdown(&mut self) -> Result<(), Error>;

    // TODO: client/registerCapability
    // TODO: client/unregisterCapability

    fn will_save_wait_until_text_document(
        &mut self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>, Error>;
}
