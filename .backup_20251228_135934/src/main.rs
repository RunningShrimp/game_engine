fn main() {
    if let Err(e) = game_engine::core::Engine::run() {
        tracing::error!(target: "main", "Engine failed to start: {}", e);
        std::process::exit(1);
    }
}
