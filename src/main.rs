fn main() {
    if let Err(e) = game_engine::core::Engine::run() {
        eprintln!("Engine failed to start: {}", e);
        std::process::exit(1);
    }
}
