//! # 悬停提示测试套件

use game_engine::tools::lsp::server::GameEngineLSP;
use tower_lsp::{Client, LanguageServer};
use tower_lsp::lsp_types::*;

async fn create_test_server() -> GameEngineLSP {
    let client = Client::new(tokio::io::empty(), tokio::io::sink());
    GameEngineLSP::new(client)
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    #[tokio::test]
    async fn test_hover_on_type() {
        let server = create_test_server().await;
        assert!(true, "Hover on type test placeholder");
    }

    #[tokio::test]
    async fn test_hover_on_function() {
        let server = create_test_server().await;
        assert!(true, "Hover on function test placeholder");
    }

    #[tokio::test]
    async fn test_hover_documentation() {
        let server = create_test_server().await;
        assert!(true, "Hover documentation test placeholder");
    }
}
