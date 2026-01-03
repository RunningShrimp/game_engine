//! # Game Engine LSP Server Binary
//!
//! Language Server Protocol server for the game engine.
//! Provides IDE support with code completion, hover info, and diagnostics.

#![cfg(feature = "lsp")]

use game_engine::tools::lsp::server::serve;
use std::env;
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("tower_lsp=info".parse()?),
        )
        .init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        process::exit(0);
    }

    if args.len() > 1 && (args[1] == "-v" || args[1] == "--version") {
        print_version();
        process::exit(0);
    }

    // Log startup
    tracing::info!("Game Engine LSP Server starting...");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Start the LSP server
    if let Err(e) = serve().await {
        tracing::error!("LSP server error: {:?}", e);
        process::exit(1);
    }

    Ok(())
}

fn print_usage() {
    println!("Game Engine LSP Server");
    println!();
    println!("USAGE:");
    println!("    game-engine-lsp [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help     Print this help message");
    println!("    -v, --version  Print version information");
    println!();
    println!("DESCRIPTION:");
    println!("    Language Server Protocol implementation for the game engine.");
    println!("    Provides intelligent code completion, hover information,");
    println!("    and real-time diagnostics for engine API usage.");
    println!();
    println!("    This server communicates via stdin/stdout using the LSP protocol.");
    println!("    It should be launched by your IDE/editor, not manually.");
    println!();
    println!("EXAMPLES:");
    println!("    In VS Code settings.json:");
    println!("    {{");
    println!("      \"gameEngine.lsp.path\": \"/path/to/game-engine-lsp\"");
    println!("    }}");
}

fn print_version() {
    println!(
        "Game Engine LSP Server version {}",
        env!("CARGO_PKG_VERSION")
    );
    println!("Part of the Game Engine project");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage_prints() {
        print_usage();
        print_version();
    }
}
