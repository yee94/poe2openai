use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub url: String,
    pub content_type: String,
    pub name: String,
    pub parsed_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolMessage {
    pub role: String,
    pub content: String,
    pub content_type: String,
    pub attachments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryRequest {
    pub version: String,
    pub r#type: String,
    pub query: Vec<ProtocolMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub user_id: String,
    pub conversation_id: String,
    pub message_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialResponse {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub text: String,
    pub allow_retry: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventType {
    Text,
    ReplaceResponse,
    Done,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    pub event: EventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<PartialResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentUploadResponse {
    pub inline_ref: Option<String>,
    pub attachment_url: Option<String>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentRequest {
    pub message_id: String,
    pub is_inline: bool,
    pub download_url: Option<String>,
    pub download_filename: Option<String>,
}

pub const URL_ATTACHMENT_TEMPLATE: &str = "URL Attachment: {attachment_name}\nContent: {content}";

pub const TEXT_ATTACHMENT_TEMPLATE: &str = "Text Attachment: {attachment_name}\nContent: {attachment_parsed_content}";

pub const IMAGE_VISION_ATTACHMENT_TEMPLATE: &str = "Image: {filename}\nDescription: {parsed_image_description}";

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub allow_attachments: Option<bool>, // Whether to allow attachments
    pub introduction_message: Option<String>, // Introduction message
    pub expand_text_attachments: Option<bool>, // Request parsed content from text attachments
    pub enable_image_comprehension: Option<bool>, // Similar for images
    pub enforce_author_role_alternation: Option<bool>, // Role alternation enforcement
    pub enable_multi_bot_chat_prompting: Option<bool>, // Multi-bot context
    pub custom_rate_card: Option<String>, // Custom rate card if applicable
}