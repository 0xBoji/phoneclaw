use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub channel: String,
    pub session_key: String,
    pub content: String,
    pub role: Role,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

impl Message {
    pub fn new(channel: &str, session_key: &str, role: Role, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel: channel.to_string(),
            session_key: session_key.to_string(),
            content: content.to_string(),
            role,
            metadata: HashMap::new(),
        }
    }
}
