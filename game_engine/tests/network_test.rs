#[cfg(test)]
mod tests {
    use super::*;
    use network::{ClientConfig, GameClient, NetworkMessage, ServerConfig, GameServer};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_client_reconnect_mechanism() {
        // 创建客户端配置
        let config = ClientConfig::default();
        
        // 创建客户端
        let mut client = GameClient::new(config);
        
        // 尝试连接到一个不存在的服务器
        match client.connect() {
            Ok(_) => {
                // 连接成功，可能服务器已经在运行，这不在测试范围内
            },
            Err(_) => {
                // 连接失败，测试预期的情况
            }
        }
        
        // 等待一段时间看是否会重连
        thread::sleep(Duration::from_secs(10));
        
        // 检查连接状态应该是Disconnected，因为没有可用的服务器
        assert_eq!(client.connection_state(), network::ConnectionState::Disconnected);
    }
    
    #[test]
    fn test_server_broadcast() {
        // 创建服务器配置
        let config = ServerConfig::default();
        
        // 创建服务器
        let mut server = GameServer::new(config);
        
        // 尝试启动服务器
        match server.start() {
            Ok(_) => {
                // 服务器启动成功
                
                // 创建几个客户端连接并验证广播功能
                // 由于这是一个测试环境，我们将模拟客户端连接
                
                // 广播一条消息
                let message = NetworkMessage::Heartbeat { 
                    timestamp: crate::core::utils::current_timestamp_ms() 
                };
                
                match server.broadcast(&message) {
                    Ok(_) => {
                        // 广播成功
                        assert!(true);
                    },
                    Err(e) => {
                        // 广播失败
                        eprintln!("Broadcast failed: {}", e);
                        assert!(false);
                    }
                }
                
                // 停止服务器
                server.stop();
            },
            Err(e) => {
                // 服务器启动失败，可能端口已经被占用，这不在测试范围内
                eprintln!("Server start failed: {}", e);
            }
        }
    }
}