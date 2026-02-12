use pocketclaw_core::types::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::sheets::SheetsClient;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    pub history: Vec<Message>,
    pub summary: Option<String>,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    storage_path: PathBuf,
    sheets_client: Option<SheetsClient>,
}

impl SessionManager {
    pub fn new(workspace: PathBuf, sheets_client: Option<SheetsClient>) -> Self {
        let storage_path = workspace.join("sessions");
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
            sheets_client,
        }
    }

    async fn load_session(&self, session_key: &str) -> Session {
        // Try to load from Google Sheets first if configured
        if let Some(client) = &self.sheets_client {
            match client.load_session(session_key).await {
                Ok(Some(session)) => {
                    // Update local file cache for redundancy
                    self.save_session_local(session_key, &session).await;
                    return session;
                }
                Ok(None) => {
                    // Sheet doesn't exist, proceed to local (maybe new session or local only)
                }
                Err(e) => {
                    tracing::error!("Failed to load session from Google Sheets: {}", e);
                    // Fallback to local
                }
            }
        }

        // Fallback to local file
        self.load_session_local(session_key).await
    }

    async fn load_session_local(&self, session_key: &str) -> Session {
        let safe_key = session_key.replace(":", "_");
        let file_path = self.storage_path.join(format!("{}.json", safe_key));
        
        if file_path.exists() {
             if let Ok(content) = fs::read_to_string(&file_path).await {
                 if let Ok(session) = serde_json::from_str(&content) {
                     return session;
                 }
             }
        }
        Session::default()
    }
    
    async fn save_session(&self, session_key: &str, session: &Session) {
        // Save locally
        self.save_session_local(session_key, session).await;
        
        // TODO: Sync summary updates to Google Sheets (requires update_summary in sheets.rs)
    }

    async fn save_session_local(&self, session_key: &str, session: &Session) {
        if !self.storage_path.exists() {
             let _ = fs::create_dir_all(&self.storage_path).await;
        }

        let safe_key = session_key.replace(":", "_");
        let file_path = self.storage_path.join(format!("{}.json", safe_key));
        
        if let Ok(content) = serde_json::to_string_pretty(session) {
            let _ = fs::write(file_path, content).await;
        }
    }

    pub async fn get_history(&self, session_key: &str) -> Vec<Message> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get(session_key) {
            return session.history.clone();
        }
        
        // Try load from disk / sheets
        let session = self.load_session(session_key).await;
        let history = session.history.clone();
        sessions.insert(session_key.to_string(), session);
        
        history
    }

    pub async fn add_message(&self, session_key: &str, message: Message) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(session_key.to_string()).or_insert_with(Session::default);
        session.history.push(message.clone());
        
        // Persist to disk
        self.save_session_local(session_key, session).await;

        // Persist to sheets (append only new message)
        if let Some(client) = &self.sheets_client {
            // We spawn this to not block the main loop latency
            let client = client.clone();
            let session_key = session_key.to_string();
            let msg = message.clone();
            tokio::spawn(async move {
                if let Err(e) = client.append_message(&session_key, &msg).await {
                    tracing::error!("Failed to append to Google Sheets: {}", e);
                }
            });
        }
    }

    pub async fn get_summary(&self, session_key: &str) -> Option<String> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get(session_key) {
            return session.summary.clone();
        }

        let session = self.load_session(session_key).await;
        let summary = session.summary.clone();
        sessions.insert(session_key.to_string(), session);
        
        summary
    }

    pub async fn set_summary(&self, session_key: &str, summary: String) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(session_key.to_string()).or_insert_with(Session::default);
        session.summary = Some(summary);
        
        self.save_session(session_key, session).await;
    }
}
