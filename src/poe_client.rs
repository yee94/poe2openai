use futures_util::Stream;
use poe_api_process::{EventResponse, PoeClient, PoeError, ProtocolMessage, QueryRequest};
use std::pin::Pin;
use tracing::{debug, error, info};
use crate::types::Message;
use std::time::Instant;

pub struct PoeClientWrapper {
    client: PoeClient,
}

impl PoeClientWrapper {
    pub fn new(model: &str, access_key: &str) -> Self {
        info!("🔑 初始化 POE 客戶端 | 模型: {}", model);
        Self {
            client: PoeClient::new(model, access_key)
        }
    }

    pub async fn stream_request(&self, query_request: QueryRequest) -> Result<Pin<Box<dyn Stream<Item = Result<EventResponse, PoeError>> + Send>>, PoeError> {
        let start_time = Instant::now();
        debug!("📤 發送串流請求 | 訊息數量: {} | 溫度設置: {:?}", 
            query_request.query.len(),
            query_request.temperature
        );

        let result = self.client.stream_request(query_request).await;
        
        match &result {
            Ok(_) => {
                let duration = start_time.elapsed();
                info!("✅ 串流請求建立成功 | 耗時: {}", 
                    crate::utils::format_duration(duration)
                );
            },
            Err(e) => {
                let duration = start_time.elapsed();
                error!("❌ 串流請求失敗 | 錯誤: {} | 耗時: {}", 
                    e,
                    crate::utils::format_duration(duration)
                );
            }
        }
        
        result
    }
}

pub fn create_query_request(messages: Vec<Message>, temperature: Option<f32>) -> QueryRequest {
    debug!("📝 創建查詢請求 | 訊息數量: {} | 溫度設置: {:?}", messages.len(), temperature);
    
    let query = messages.into_iter().map(|msg| {
        debug!("🔄 轉換訊息 | 角色: {} | 內容長度: {}", 
            msg.role,
            crate::utils::format_bytes_length(msg.content.len())
        );
        
        ProtocolMessage {
            role: convert_role(&msg.role),
            content: msg.content,
            content_type: "text/markdown".to_string(),
        }
    }).collect();

    QueryRequest {
        version: "1".to_string(),
        r#type: "query".to_string(),
        query,
        temperature,
        user_id: "".to_string(),
        conversation_id: "".to_string(),
        message_id: "".to_string(),
    }
}

fn convert_role(role: &str) -> String {
    let converted = if role == "assistant" {
        "bot".to_string()
    } else if role == "system" {
        "user".to_string()
    } else {
        role.to_string()
    };
    debug!("🔄 角色轉換: {} -> {}", role, converted);
    converted
}