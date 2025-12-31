//! Asset Pipeline 集成测试

#[cfg(feature = "cli")]
#[test]
fn test_optimize_command_parsing() {
    use game_engine::tools::cli::GameEngineCli;

    // 测试基础optimize命令
    let cli = GameEngineCli::try_parse_from([
        "game-engine",
        "optimize",
        "./assets",
        "-o",
        "./assets_optimized",
    ]);

    assert!(cli.is_ok());
}

#[cfg(feature = "cli")]
#[test]
fn test_analyze_command_parsing() {
    use game_engine::tools::cli::GameEngineCli;

    let cli = GameEngineCli::try_parse_from(["game-engine", "analyze", "./assets"]);

    assert!(cli.is_ok());
}

#[cfg(feature = "cli")]
#[test]
fn test_bundle_command_parsing() {
    use game_engine::tools::cli::GameEngineCli;

    let cli = GameEngineCli::try_parse_from([
        "game-engine",
        "bundle",
        "./assets",
        "-o",
        "game.pak",
    ]);

    assert!(cli.is_ok());
}
