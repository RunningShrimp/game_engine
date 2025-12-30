//! 消息定义
//!
//! 定义微内核架构中的消息类型和处理机制。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(u64);

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageId {
    pub fn new() -> Self {
        Self(std::sync::atomic::AtomicU64::new(1).fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }

    pub fn from(id: u64) -> Self {
        Self(id)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Broadcast,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub type_: String,
    pub data: Vec<u8>,
}

impl MessagePayload {
    pub fn new(type_: String, data: Vec<u8>) -> Self {
        Self { type_, data }
    }

    #[cfg(feature = "message-optimization")]
    pub fn serialize<T: Serialize>(value: &T) -> Result<Self, Box<dyn std::error::Error>> {
        let data = bincode::serialize(value)?;
        Ok(Self::new(std::any::type_name::<T>().to_string(), data))
    }

    #[cfg(feature = "message-optimization")]
    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<T, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(&self.data)?)
    }

    #[cfg(not(feature = "message-optimization"))]
    pub fn serialize<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let data = serde_json::to_vec(value)?;
        Ok(Self::new(std::any::type_name::<T>().to_string(), data))
    }

    #[cfg(not(feature = "message-optimization"))]
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.data)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub source: super::ServiceId,
    pub target: Option<super::ServiceId>,
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub timestamp: std::time::Instant,
}

impl Message {
    pub fn new(
        source: super::ServiceId,
        target: Option<super::ServiceId>,
        message_type: MessageType,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: MessageId::new(),
            source,
            target,
            message_type,
            payload,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn request(
        source: super::ServiceId,
        target: super::ServiceId,
        payload: MessagePayload,
    ) -> Self {
        Self::new(source, Some(target), MessageType::Request, payload)
    }

    pub fn response(
        source: super::ServiceId,
        target: super::ServiceId,
        payload: MessagePayload,
    ) -> Self {
        Self::new(source, Some(target), MessageType::Response, payload)
    }

    pub fn notification(source: super::ServiceId, payload: MessagePayload) -> Self {
        Self::new(source, None, MessageType::Notification, payload)
    }

    pub fn broadcast(source: super::ServiceId, payload: MessagePayload) -> Self {
        Self::new(source, None, MessageType::Broadcast, payload)
    }

    pub fn with_reply_to(mut self, reply_to: super::ServiceId) -> Self {
        self.target = Some(reply_to);
        self
    }
}

#[derive(Debug)]
pub struct Request {
    pub message: Message,
    pub reply_channel: tokio::sync::oneshot::Sender<Response>,
}

impl Request {
    pub fn new(message: Message) -> (Self, tokio::sync::oneshot::Receiver<Response>) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        (
            Self {
                message,
                reply_channel: tx,
            },
            rx,
        )
    }
}

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

#[derive(Debug, Clone, Default)]
pub struct MessageHeaders {
    headers: HashMap<String, String>,
}

impl MessageHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.headers.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ServiceId;
    use super::*;

    #[test]
    fn test_message_id() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_message_creation() {
        let source = ServiceId::new("source_service");
        let target = ServiceId::new("target_service");
        let payload = MessagePayload::new("test".to_string(), vec![1, 2, 3]);

        let message = Message::request(source.clone(), target, payload);
        assert_eq!(message.source, source);
        assert_eq!(message.message_type, MessageType::Request);
    }

    #[test]
    fn test_payload_serialization() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestData {
            value: i32,
        }

        let data = TestData { value: 42 };
        let payload =
            MessagePayload::serialize(&data).expect("Failed to serialize TestData payload");
        let deserialized: TestData =
            payload.deserialize().expect("Failed to deserialize TestData payload");

        assert_eq!(deserialized, data);
    }
}
