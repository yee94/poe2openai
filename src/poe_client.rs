use futures_util::Stream;
use poe_api_process::{EventResponse, PoeClient, PoeError, ProtocolMessage, QueryRequest};
use std::pin::Pin;

use crate::types::Message;

pub struct PoeClientWrapper {
    client: PoeClient,
}

impl PoeClientWrapper {
    pub fn new(model: &str, access_key: &str) -> Self {
        Self {
            client: PoeClient::new(model, access_key),
        }
    }

    pub async fn stream_request(&self, query_request: QueryRequest) -> Result<Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>, PoeError> {
        self.client.stream_request(query_request).await
    }
}

pub fn create_query_request(messages: Vec<Message>, temperature: Option<f32>) -> QueryRequest {
    QueryRequest {
        version: "1".to_string(),
        r#type: "query".to_string(),
        query: messages.into_iter().map(|msg| ProtocolMessage {
            role: msg.role,
            content: msg.content,
            content_type: "text/markdown".to_string(),
        }).collect(),
        temperature,
        user_id: "".to_string(),
        conversation_id: "".to_string(),
        message_id: "".to_string(),
    }
}