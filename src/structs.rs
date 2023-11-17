use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: String,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

#[derive(Serialize, Deserialize)]
pub struct UserMessage {
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<UserMessage>,
    pub max_tokens: u32,
}