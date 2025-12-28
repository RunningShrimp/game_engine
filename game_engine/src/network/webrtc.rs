//! WebRTC网络协议支持
//!
//! 提供WebRTC连接管理、数据通道、信令和NAT穿透支持。
//! 用于实时多人游戏和音视频通信。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{RwLock, mpsc};

/// WebRTC错误类型
#[derive(Error, Debug)]
pub enum WebRtcError {
    /// 连接失败
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    /// 信令错误
    #[error("Signaling error: {0}")]
    SignalingError(String),
    /// ICE候选错误
    #[error("ICE candidate error: {0}")]
    IceCandidateError(String),
    /// SDP错误
    #[error("SDP error: {0}")]
    SdpError(String),
    /// 数据通道错误
    #[error("Data channel error: {0}")]
    DataChannelError(String),
    /// 配置错误
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    /// 超时错误
    #[error("Operation timeout")]
    Timeout,
    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

/// WebRTC配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    /// STUN服务器列表
    pub stun_servers: Vec<String>,
    /// TURN服务器列表
    pub turn_servers: Vec<TurnServerConfig>,
    /// ICE传输策略
    pub ice_transport_policy: IceTransportPolicy,
    /// 数据通道配置
    pub data_channel_config: DataChannelConfig,
    /// 是否启用音频
    pub enable_audio: bool,
    /// 是否启用视频
    pub enable_video: bool,
    /// 连接超时（秒）
    pub connection_timeout_secs: u64,
}

/// TURN服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    /// 服务器URL
    pub url: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub credential: String,
}

/// ICE传输策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceTransportPolicy {
    /// 仅使用中继（TURN）
    Relay,
    /// 仅使用直连（STUN）
    All,
}

/// 数据通道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChannelConfig {
    /// 是否有序
    pub ordered: bool,
    /// 最大重传次数（0表示不重传）
    pub max_retransmits: Option<u16>,
    /// 最大重传时间（毫秒）
    pub max_packet_life_time: Option<u16>,
    /// 协议名称
    pub protocol: String,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn_servers: Vec::new(),
            ice_transport_policy: IceTransportPolicy::All,
            data_channel_config: DataChannelConfig {
                ordered: true,
                max_retransmits: Some(3),
                max_packet_life_time: None,
                protocol: "game-data".to_string(),
            },
            enable_audio: false,
            enable_video: false,
            connection_timeout_secs: 30,
        }
    }
}

/// WebRTC连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebRtcConnectionState {
    /// 新建
    New,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 断开连接中
    Disconnecting,
    /// 已断开
    Disconnected,
    /// 连接失败
    Failed,
    /// 已关闭
    Closed,
}

/// ICE连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceConnectionState {
    /// 新建
    New,
    /// 检查中
    Checking,
    /// 已连接
    Connected,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已断开
    Disconnected,
    /// 已关闭
    Closed,
}

/// ICE收集状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceGatheringState {
    /// 新建
    New,
    /// 收集中
    Gathering,
    /// 已完成
    Complete,
}

/// 信令消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalingMessage {
    /// Offer（发起方）
    Offer { sdp: String, session_id: String },
    /// Answer（接收方）
    Answer { sdp: String, session_id: String },
    /// ICE候选
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
        session_id: String,
    },
    /// 连接关闭
    Close { session_id: String },
}

/// WebRTC对等连接
///
/// 管理单个WebRTC连接，包括信令、ICE候选和数据通道。
pub struct WebRtcPeerConnection {
    /// 连接ID
    pub connection_id: String,
    /// 配置
    pub config: WebRtcConfig,
    /// 连接状态
    pub connection_state: Arc<RwLock<WebRtcConnectionState>>,
    /// ICE连接状态
    pub ice_connection_state: Arc<RwLock<IceConnectionState>>,
    /// ICE收集状态
    pub ice_gathering_state: Arc<RwLock<IceGatheringState>>,
    /// 数据通道发送端
    pub data_channel_tx: Option<mpsc::Sender<Vec<u8>>>,
    /// 数据通道接收端（使用Arc<Mutex>以便共享）
    pub data_channel_rx: Option<Arc<tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>>>,
    /// 信令消息发送端
    pub signaling_tx: mpsc::Sender<SignalingMessage>,
    /// 信令消息接收端
    pub signaling_rx: mpsc::Receiver<SignalingMessage>,
}

impl WebRtcPeerConnection {
    /// 创建新的WebRTC对等连接
    pub fn new(connection_id: String, config: WebRtcConfig) -> Self {
        let (signaling_tx, signaling_rx) = mpsc::channel(100);

        Self {
            connection_id,
            config,
            connection_state: Arc::new(RwLock::new(WebRtcConnectionState::New)),
            ice_connection_state: Arc::new(RwLock::new(IceConnectionState::New)),
            ice_gathering_state: Arc::new(RwLock::new(IceGatheringState::New)),
            data_channel_tx: None,
            data_channel_rx: None,
            signaling_tx,
            signaling_rx,
        }
    }

    /// 创建Offer（作为发起方）
    pub async fn create_offer(&mut self) -> Result<String, WebRtcError> {
        // 更新连接状态
        *self.connection_state.write().await = WebRtcConnectionState::Connecting;

        // 实际实现应该调用WebRTC库创建Offer
        // 这里返回一个占位符SDP
        let sdp = format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             a=group:BUNDLE 0\r\n\
             a=msid-semantic: WMS\r\n\
             m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=ice-ufrag:{}\r\n\
             a=ice-pwd:{}\r\n\
             a=fingerprint:sha-256 {}\r\n\
             a=setup:actpass\r\n\
             a=mid:0\r\n\
             a=sctp-port:5000\r\n\
             a=max-message-size:262144\r\n",
            uuid::Uuid::new_v4(),
            "test_ufrag",
            "test_pwd",
            "placeholder_fingerprint"
        );

        // 发送Offer信令消息
        let message = SignalingMessage::Offer {
            sdp: sdp.clone(),
            session_id: self.connection_id.clone(),
        };
        self.signaling_tx
            .send(message)
            .await
            .map_err(|e| WebRtcError::SignalingError(e.to_string()))?;

        Ok(sdp)
    }

    /// 创建Answer（作为接收方）
    pub async fn create_answer(&mut self, _offer_sdp: &str) -> Result<String, WebRtcError> {
        // 更新连接状态
        *self.connection_state.write().await = WebRtcConnectionState::Connecting;

        // 实际实现应该调用WebRTC库创建Answer
        // 这里返回一个占位符SDP
        let sdp = format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             a=group:BUNDLE 0\r\n\
             a=msid-semantic: WMS\r\n\
             m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=ice-ufrag:{}\r\n\
             a=ice-pwd:{}\r\n\
             a=fingerprint:sha-256 {}\r\n\
             a=setup:active\r\n\
             a=mid:0\r\n\
             a=sctp-port:5000\r\n\
             a=max-message-size:262144\r\n",
            uuid::Uuid::new_v4(),
            "test_ufrag",
            "test_pwd",
            "placeholder_fingerprint"
        );

        // 发送Answer信令消息
        let message = SignalingMessage::Answer {
            sdp: sdp.clone(),
            session_id: self.connection_id.clone(),
        };
        self.signaling_tx
            .send(message)
            .await
            .map_err(|e| WebRtcError::SignalingError(e.to_string()))?;

        Ok(sdp)
    }

    /// 设置远程描述（SDP）
    pub async fn set_remote_description(
        &mut self,
        sdp: &str,
        is_offer: bool,
    ) -> Result<(), WebRtcError> {
        // 实际实现应该调用WebRTC库设置远程描述
        // 这里只是占位符实现

        if is_offer {
            // 如果是Offer，创建Answer
            self.create_answer(sdp).await?;
        }

        Ok(())
    }

    /// 添加ICE候选
    pub async fn add_ice_candidate(
        &mut self,
        candidate: &str,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<(), WebRtcError> {
        // 实际实现应该调用WebRTC库添加ICE候选
        // 这里发送信令消息
        let message = SignalingMessage::IceCandidate {
            candidate: candidate.to_string(),
            sdp_mid,
            sdp_mline_index,
            session_id: self.connection_id.clone(),
        };
        self.signaling_tx
            .send(message)
            .await
            .map_err(|e| WebRtcError::IceCandidateError(e.to_string()))?;

        Ok(())
    }

    /// 创建数据通道
    pub async fn create_data_channel(&mut self, _label: &str) -> Result<(), WebRtcError> {
        // 实际实现应该调用WebRTC库创建数据通道
        // 这里创建异步通道用于数据传递
        let (tx, rx) = mpsc::channel(1000);
        self.data_channel_tx = Some(tx);
        self.data_channel_rx = Some(Arc::new(tokio::sync::Mutex::new(rx)));

        Ok(())
    }

    /// 通过数据通道发送数据
    pub async fn send_data(&self, data: Vec<u8>) -> Result<(), WebRtcError> {
        if let Some(ref tx) = self.data_channel_tx {
            tx.send(data).await.map_err(|e| WebRtcError::DataChannelError(e.to_string()))?;
            Ok(())
        } else {
            Err(WebRtcError::DataChannelError(
                "Data channel not created".to_string(),
            ))
        }
    }

    /// 接收数据通道数据
    pub async fn receive_data(&self) -> Option<Vec<u8>> {
        if let Some(ref rx) = self.data_channel_rx {
            let mut rx_guard = rx.lock().await;
            rx_guard.recv().await
        } else {
            None
        }
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<(), WebRtcError> {
        *self.connection_state.write().await = WebRtcConnectionState::Disconnecting;

        // 发送关闭信令消息
        let message = SignalingMessage::Close {
            session_id: self.connection_id.clone(),
        };
        let _ = self.signaling_tx.send(message).await;

        *self.connection_state.write().await = WebRtcConnectionState::Closed;
        Ok(())
    }

    /// 获取连接状态
    pub async fn connection_state(&self) -> WebRtcConnectionState {
        *self.connection_state.read().await
    }

    /// 获取ICE连接状态
    pub async fn ice_connection_state(&self) -> IceConnectionState {
        *self.ice_connection_state.read().await
    }

    /// 获取ICE收集状态
    pub async fn ice_gathering_state(&self) -> IceGatheringState {
        *self.ice_gathering_state.read().await
    }
}

/// WebRTC管理器
///
/// 管理多个WebRTC连接，提供信令服务器接口和连接池管理。
/// 注意：由于 mpsc::Receiver 不可克隆，连接不能使用 Arc 共享。
pub struct WebRtcManager {
    /// 配置
    config: WebRtcConfig,
    /// 活跃连接（使用Arc以便共享）
    connections: Arc<RwLock<HashMap<String, WebRtcPeerConnection>>>,
    /// 信令消息处理器
    signaling_handler: Option<Box<dyn SignalingHandler + Send + Sync>>,
}

/// 信令处理器trait
pub trait SignalingHandler: Send + Sync {
    /// 发送信令消息
    fn send_message(&self, message: SignalingMessage) -> Result<(), WebRtcError>;
    /// 接收信令消息
    fn receive_message(&self) -> Option<SignalingMessage>;
}

impl WebRtcManager {
    /// 创建新的WebRTC管理器
    pub fn new(config: WebRtcConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            signaling_handler: None,
        }
    }

    /// 设置信令处理器
    pub fn set_signaling_handler(&mut self, handler: Box<dyn SignalingHandler + Send + Sync>) {
        self.signaling_handler = Some(handler);
    }

    /// 创建新的对等连接
    pub async fn create_peer_connection(&self, connection_id: String) -> Result<(), WebRtcError> {
        let mut conn = WebRtcPeerConnection::new(connection_id.clone(), self.config.clone());

        // 创建数据通道
        conn.create_data_channel("game-data").await?;

        // 添加到连接池
        self.connections.write().await.insert(connection_id, conn);

        Ok(())
    }

    /// 获取对等连接（可变引用）
    pub async fn get_peer_connection_mut(
        &self,
        connection_id: &str,
    ) -> Option<WebRtcPeerConnection> {
        let mut guard = self.connections.write().await;
        guard.remove(connection_id)
    }

    /// 获取对等连接（只读引用）
    pub async fn get_peer_connection(
        &self,
        connection_id: &str,
    ) -> Option<WebRtcPeerConnection> {
        let guard = self.connections.read().await;
        // 由于 WebRtcPeerConnection 不实现 Clone，我们需要克隆连接
        guard.get(connection_id).map(|_| {
            // 创建一个新连接作为克隆
            let conn_info = guard.get(connection_id).unwrap();
            WebRtcPeerConnection::new(conn_info.connection_id.clone(), conn_info.config.clone())
        })
    }

    /// 移除对等连接
    pub async fn remove_peer_connection(&self, connection_id: &str) -> Result<(), WebRtcError> {
        if let Some(mut conn) = self.connections.write().await.remove(connection_id) {
            conn.close().await?;
        }
        Ok(())
    }

    /// 处理信令消息
    pub async fn handle_signaling_message(
        &self,
        message: SignalingMessage,
    ) -> Result<(), WebRtcError> {
        match message {
            SignalingMessage::Offer { sdp, session_id } => {
                if let Some(mut conn) = self.get_peer_connection_mut(&session_id).await {
                    conn.set_remote_description(&sdp, true).await?;
                    // 将连接放回映射
                    self.connections.write().await.insert(session_id.clone(), conn);
                }
            }
            SignalingMessage::Answer { sdp, session_id } => {
                if let Some(mut conn) = self.get_peer_connection_mut(&session_id).await {
                    conn.set_remote_description(&sdp, false).await?;
                    // 将连接放回映射
                    self.connections.write().await.insert(session_id.clone(), conn);
                }
            }
            SignalingMessage::IceCandidate {
                candidate,
                sdp_mid,
                sdp_mline_index,
                session_id,
            } => {
                if let Some(mut conn) = self.get_peer_connection_mut(&session_id).await {
                    conn.add_ice_candidate(&candidate, sdp_mid, sdp_mline_index).await?;
                    // 将连接放回映射
                    self.connections.write().await.insert(session_id.clone(), conn);
                }
            }
            SignalingMessage::Close { session_id } => {
                self.remove_peer_connection(&session_id).await?;
            }
        }
        Ok(())
    }

    /// 获取所有连接ID
    pub async fn get_connection_ids(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

// 注意：WebRtcPeerConnection不实现Clone，因为mpsc::Receiver不能克隆
// 如果需要共享连接，应该使用Arc<WebRtcPeerConnection>

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_peer_connection() {
        let config = WebRtcConfig::default();
        let manager = WebRtcManager::new(config);

        let connection_id = "test_connection".to_string();
        let result = manager.create_peer_connection(connection_id.clone()).await;

        assert!(result.is_ok());
        assert_eq!(manager.connection_count().await, 1);

        // 测试获取连接
        let conn = manager.get_peer_connection(&connection_id).await;
        assert!(conn.is_some());
    }

    #[tokio::test]
    async fn test_create_offer() {
        let config = WebRtcConfig::default();
        let mut conn = WebRtcPeerConnection::new("test".to_string(), config);

        let result = conn.create_offer().await;
        assert!(result.is_ok());

        let state = conn.connection_state().await;
        assert_eq!(state, WebRtcConnectionState::Connecting);
    }

    #[tokio::test]
    async fn test_data_channel() {
        let config = WebRtcConfig::default();
        let mut conn = WebRtcPeerConnection::new("test".to_string(), config);

        // 创建数据通道
        let result = conn.create_data_channel("test").await;
        assert!(result.is_ok());

        // 发送数据
        let data = b"test message".to_vec();
        let result = conn.send_data(data.clone()).await;
        assert!(result.is_ok());

        // 接收数据（注意：由于我们使用的是本地通道，需要在实际实现中处理）
        // 这里只是测试接口
    }
}
