use log::info;

use std::{
    fs::File,
    io::{self, BufReader},
};

use lspine::{
    InitializeRequestParams, InitializeResult, LSPError, LanguageServer, Runner,
    ServerCapabilities, ServerInfo,
};

use simplelog::{ConfigBuilder, WriteLogger};

struct BidhuLsp;

impl LanguageServer for BidhuLsp {
    fn initialize(
        &mut self,
        params: &InitializeRequestParams,
    ) -> Result<InitializeResult, LSPError> {
        info!("client -> {:?}", params);
        let result = InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: false,
            },
            server_info: ServerInfo {
                name: String::from("bidhulsp"),
                version: String::from("0.0.0.0.0.1"),
            },
        };
        info!("server -> {:?}", result.server_info);

        Ok(result)
    }

    fn shutdown(&mut self) -> Result<(), LSPError> {
        Ok(())
    }
}

fn main() {
    let log_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("app.log")
        .expect("ERROR: failed to open the log file");

    WriteLogger::init(
        simplelog::LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_offset_to_local()
            .unwrap()
            .build(),
        log_file,
    )
    .expect("ERROR: failed to set up logger");

    info!("Starting the lsp server");
    Runner::new(BidhuLsp {}, BufReader::new(io::stdin()), io::stdout())
        .run()
        .unwrap();
}
