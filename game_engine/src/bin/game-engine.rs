//! # Game Engine CLI Binary
//!
//! Command-line interface for the game engine project scaffolding tool.
//!
//! ## Usage
//!
//! ```bash
//! # Create a new project
//! game-engine new my-game --template basic
//!
//! # List templates
//! game-engine template list
//!
//! # Get info
//! game-engine info
//! ```

use game_engine::tools::cli::GameEngineCli;

fn main() {
    // Parse CLI arguments
    let cli = match GameEngineCli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Print error message and exit
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    };

    // Set verbosity level based on -v flags
    if let Err(e) = init_logging(cli.verbose) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    // Run the command
    if let Err(e) = cli.run() {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: u8) -> Result<(), Box<dyn std::error::Error>> {
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    let level = match verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).with_target(false).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
