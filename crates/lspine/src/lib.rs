mod error;
pub mod model;
mod runner;
pub mod transport;

pub use error::*;
pub use model::*;
pub use runner::*;
pub use transport::*;

pub trait LanguageServer {
    fn initialize(
        &mut self,
        params: &InitializeRequestParams,
    ) -> Result<InitializeResult, LSPError>;

    fn shutdown(&mut self) -> Result<(), LSPError>;

    fn notification(&mut self, _method: &str) {}

    // TODO: client/registerCapability
    // TODO: client/unregisterCapability

    // Document Syncronization
    //

    fn did_open_text_docuement(&mut self, _params: DidOpenTextDocumentParams) {
        self.notification("textDocument/didOpen");
    }

    fn did_change_text_docuement(&mut self, _params: DidChangeTextDocumentParams) {
        self.notification("textDocument/didChange");
    }

    fn will_save_text_docuement(&mut self, _params: WillSaveTextDocumentParams) {
        self.notification("textDocument/willSave");
    }

    fn will_save_wait_until_text_document(
        &mut self,
        _params: &WillSaveTextDocumentParams,
    ) -> Result<WillSaveWaitUntilResult, LSPError> {
        Ok(WillSaveWaitUntilResult(None))
    }

    fn did_save_text_docuement(&mut self, _params: DidSaveTextDocumentParams) {
        self.notification("textDocument/didSave");
    }

    fn did_close_text_docuement(&mut self, _params: DidCloseTextDocumentParams) {
        self.notification("textDocument/didClose");
    }

    // Language Features
    //

    fn go_to_declaration(
        &mut self,
        _params: DeclarationParams,
    ) -> Result<Option<DeclarationResult>, LSPError> {
        Ok(None)
    }

    fn go_to_definition(
        &mut self,
        _params: DefinitionParams,
    ) -> Result<Option<DefinitionResult>, LSPError> {
        Ok(None)
    }

    fn go_to_type_definition(
        &mut self,
        _params: TypeDefinitionParams,
    ) -> Result<Option<TypeDefinitionResult>, LSPError> {
        Ok(None)
    }

    // Workspace Features
    //

    fn did_rename_files_workspace(&mut self, _params: RenameFilesParams) {
        self.notification("workspace/didRenameFiles");
    }
}
