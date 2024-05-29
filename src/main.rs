mod analysis;
mod backend;
mod model;
mod utils;
mod workspace;

use backend::Backend;
use clap::Parser;
use tower_lsp::{LspService, Server};

#[derive(Parser)]
#[command(version)]
struct Args {
    // TODO: Add host/port for running as TCP server or client
}

#[tokio::main]
async fn main() {
    let _args = Args::parse();

    let in_stream = tokio::io::stdin();
    let out_stream = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(in_stream, out_stream, socket).serve(service).await;
}
