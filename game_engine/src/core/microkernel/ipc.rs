//! IPC（进程间通信）通道
//!
//! 提供服务间通信的机制。

use std::collections::HashMap;

use tokio::sync::{mpsc, oneshot};

pub use super::{Message, MessagePayload, ServiceId};

#[derive(Debug, Clone)]
pub struct Response {
    pub message: Message,
    pub success: bool,
    pub error: Option<String>,
}

impl Response {
    pub fn success(message: Message) -> Self {
        Self {
            message,
            success: true,
            error: None,
        }
    }

    pub fn error(message: Message, error: String) -> Self {
        Self {
            message,
            success: false,
            error: Some(error),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub message: Message,
    pub reply_channel: oneshot::Sender<Response>,
}

impl Request {
    pub fn new(message: Message) -> (Self, oneshot::Receiver<Response>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                message,
                reply_channel: tx,
            },
            rx,
        )
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum IpcError {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Receive failed: {0}")]
    ReceiveFailed(String),

    #[error("Timeout")]
    Timeout,

    #[error("Target service not found: {0}")]
    ServiceNotFound(String),
}

pub struct IpcChannel {
    service_id: ServiceId,
    sender: mpsc::UnboundedSender<Message>,
    request_handler: tokio::sync::RwLock<Option<RequestHandler>>,
}

struct RequestHandler {
    pending_requests: tokio::sync::RwLock<HashMap<super::MessageId, oneshot::Sender<Response>>>,
}

impl RequestHandler {
    fn new() -> Self {
        Self {
            pending_requests: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// 注册响应通道，等待特定消息的响应
    pub async fn register(&self, id: super::MessageId, sender: oneshot::Sender<Response>) {
        let mut pending = self.pending_requests.write().await;
        pending.insert(id, sender);
    }

    /// 检查是否有待处理的请求
    pub fn has_pending_request(&self, id: &super::MessageId) -> bool {
        let pending = self.pending_requests.blocking_read();
        pending.contains_key(id)
    }

    /// 获取待处理请求数量
    pub fn pending_count(&self) -> usize {
        let pending = self.pending_requests.blocking_read();
        pending.len()
    }

    async fn unregister(&self, id: &super::MessageId) -> Option<oneshot::Sender<Response>> {
        let mut pending = self.pending_requests.write().await;
        pending.remove(id)
    }

    async fn complete(&self, id: &super::MessageId, response: Response) -> bool {
        let sender = self.unregister(id).await;
        if let Some(tx) = sender {
            tx.send(response).is_ok()
        } else {
            false
        }
    }
}

impl IpcChannel {
    pub fn new(service_id: ServiceId) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let handler = RequestHandler::new();

        let channel = Self {
            service_id,
            sender,
            request_handler: tokio::sync::RwLock::new(Some(handler)),
        };

        tokio::spawn(Self::receive_loop(channel.clone(), receiver));

        channel
    }

    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
    }

    pub async fn send(&self, message: Message) -> Result<Option<Message>, IpcError> {
        self.sender.send(message.clone()).map_err(|_| IpcError::ChannelClosed)?;

        if message.message_type == super::MessageType::Request {
            Ok(None)
        } else {
            Ok(Some(message))
        }
    }

    pub async fn request(&self, message: Message) -> Result<Response, IpcError> {
        let (_request, receiver) = Request::new(message.clone());

        self.sender.send(message).map_err(|_| IpcError::ChannelClosed)?;

        tokio::time::timeout(std::time::Duration::from_secs(5), receiver)
            .await
            .map_err(|_| IpcError::Timeout)?
            .map_err(|_| IpcError::ChannelClosed)
    }

    pub async fn try_recv(&self) -> Result<Option<Message>, IpcError> {
        Ok(None)
    }

    async fn receive_loop(channel: IpcChannel, mut receiver: mpsc::UnboundedReceiver<Message>) {
        while let Some(message) = receiver.recv().await {
            let message_id = message.id.clone();

            let response = match message.message_type {
                super::MessageType::Request => {
                    let handler_guard = channel.request_handler.read().await;
                    if let Some(handler) = handler_guard.as_ref()
                        && let Some(pending) = handler.unregister(&message_id).await
                    {
                        let response = Response::success(message.clone());
                        let _ = pending.send(response);
                    }
                    None
                }
                super::MessageType::Response => Some(message),
                super::MessageType::Notification | super::MessageType::Broadcast => Some(message),
                super::MessageType::Error => Some(message),
            };

            if let Some(response_msg) = response {
                let handler_guard = channel.request_handler.read().await;
                if let Some(handler) = handler_guard.as_ref() {
                    let response = Response::success(response_msg.clone());
                    let _ = handler.complete(&message_id, response).await;
                }
            }
        }
    }
}

impl Clone for IpcChannel {
    fn clone(&self) -> Self {
        Self {
            service_id: self.service_id.clone(),
            sender: self.sender.clone(),
            request_handler: tokio::sync::RwLock::new(None),
        }
    }
}

pub struct MessageBus {
    subscribers: tokio::sync::RwLock<HashMap<ServiceId, mpsc::UnboundedSender<Message>>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            subscribers: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe(&self, service_id: ServiceId) -> mpsc::UnboundedReceiver<Message> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(service_id, tx);
        rx
    }

    pub async fn unsubscribe(&self, service_id: &ServiceId) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.remove(service_id);
    }

    pub async fn publish(&self, message: Message) -> Result<usize, IpcError> {
        let subscribers = self.subscribers.read().await;
        let mut sent = 0;

        for (id, sender) in subscribers.iter() {
            if Some(id) == message.target.as_ref() && sender.send(message.clone()).is_ok() {
                sent += 1;
            }
        }

        Ok(sent)
    }

    pub async fn broadcast(&self, message: Message) -> Result<usize, IpcError> {
        let subscribers = self.subscribers.read().await;
        let mut sent = 0;

        for sender in subscribers.values() {
            if sender.send(message.clone()).is_ok() {
                sent += 1;
            }
        }

        Ok(sent)
    }

    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ipc_channel_creation() {
        let service_id = ServiceId::new("test_service");
        let channel = IpcChannel::new(service_id);
        assert_eq!(channel.service_id().as_str(), "test_service");
    }

    #[tokio::test]
    async fn test_message_bus() {
        let bus = MessageBus::new();
        let service_id = ServiceId::new("test_service");

        let mut rx = bus.subscribe(service_id.clone()).await;
        assert_eq!(bus.subscriber_count().await, 1);

        let message = Message::notification(
            service_id.clone(),
            MessagePayload::new("test".to_string(), vec![]),
        );

        let _ = bus.publish(message).await;
        let received = rx.recv().await;
        assert!(received.is_some());
    }
}
