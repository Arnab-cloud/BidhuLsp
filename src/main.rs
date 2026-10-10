mod rpc;

use log::{error, info};

use std::{
    fs::File,
    io::{self, BufReader, Write},
};

use simplelog::{ConfigBuilder, WriteLogger};

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

    let mut stdin = BufReader::new(io::stdin());
    loop {
        match rpc::read_message(&mut stdin) {
            Ok(msg) => {
                let message = &rpc::parse_message(&msg);
                match message {
                    rpc::model::Message::MRequest(request) => match request {
                        rpc::model::Request::Initialize(intialize_request) => {
                            info!("client -> {:?}", intialize_request.method);
                            let response = rpc::model::InitializeResponse {
                                jsonrpc: String::from("2.0"),
                                id: 1,
                                result: rpc::model::IntializeResult {
                                    capabilities: rpc::model::ServerCapabilities {
                                        hover_provider: false,
                                    },
                                    server_info: rpc::model::ServerInfo {
                                        name: String::from("bidhulsp"),
                                        version: String::from("0.0.0.0.0.1"),
                                    },
                                },
                            };
                            info!("server -> {:?}", response.result.server_info);
                            let msg = rpc::create_rpc_message(response).unwrap();
                            info!("{}", msg);
                            print!("{}", msg);
                            io::stdout().flush().unwrap();
                        }
                    },
                    rpc::model::Message::MNotification(val) => info!("{:?}", val.get("method")),
                    rpc::model::Message::MResponse => todo!(),
                }
            }
            Err(_) => error!("ERROR: Failed to parse the message"),
        }
    }
}
