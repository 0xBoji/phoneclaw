use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
}

pub struct GroqTranscriber {
    api_key: String,
    api_base: String,
    client: reqwest::Client,
}

impl GroqTranscriber {
    pub fn new(api_key: String) -> Self {
        info!("Creating Groq transcriber");
        Self {
            api_key,
            api_base: "https://api.groq.com/openai/v1".to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    pub fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn transcribe(&self, audio_file_path: &str) -> anyhow::Result<TranscriptionResponse> {
        info!(audio_file = audio_file_path, "Starting transcription");

        let path = Path::new(audio_file_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.ogg")
            .to_string();

        let file_bytes = tokio::fs::read(audio_file_path).await?;
        debug!(
            size_bytes = file_bytes.len(),
            file_name = %file_name,
            "Audio file read"
        );

        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str("audio/ogg")?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("model", "whisper-large-v3")
            .text("response_format", "json");

        let url = format!("{}/audio/transcriptions", self.api_base);
        debug!(url = %url, "Sending transcription request to Groq API");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            error!(status = %status, response = %body, "Groq API error");
            anyhow::bail!("API error (status {}): {}", status, body);
        }

        let result: TranscriptionResponse = serde_json::from_str(&body)?;
        info!(
            text_length = result.text.len(),
            language = ?result.language,
            "Transcription completed"
        );

        Ok(result)
    }
}
