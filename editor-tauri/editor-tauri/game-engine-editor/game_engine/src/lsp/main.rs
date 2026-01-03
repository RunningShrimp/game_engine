//! Game Engine Language Server Protocol (LSP) Server
//!
//! Provides intelligent code completion, go-to-definition,
//! hover information, and other IDE features for game engine projects.

use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{error, info};
use tracing_subscriber;

mod server;
mod api_index;
mod symbol_info;
mod completion;

use server::GameEngineServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    info!("Starting Game Engine LSP Server");

    // Create stdin/stdout streams for LSP communication
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create the LSP service
    let (service, socket) = LspService::build(|client| GameEngineServer::new(client)).finish()?;

    // Run the server
    Server::new(stdin, stdout)
        .serve(socket)
        .await?;

    Ok(())
}
