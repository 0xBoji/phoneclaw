use crate::{GenerationOptions, GenerationResponse, LLMProvider, ProviderError};
use async_trait::async_trait;
use phoneclaw_core::types::Message;
use std::sync::Arc;
use std::time::Duration;

/// Lightweight reliability wrapper for unstable mobile networks.
/// Retries transient failures with bounded exponential backoff.
pub struct ReliableProvider {
    inner: Arc<dyn LLMProvider>,
    max_retries: u32,
    base_backoff_ms: u64,
}

impl ReliableProvider {
    pub fn new(inner: Arc<dyn LLMProvider>, max_retries: u32, base_backoff_ms: u64) -> Self {
        Self {
            inner,
            max_retries,
            base_backoff_ms: base_backoff_ms.max(100),
        }
    }

    fn is_retryable(err: &ProviderError) -> bool {
        match err {
            ProviderError::NetworkError(_) => true,
            ProviderError::ApiError(message) => {
                let lower = message.to_lowercase();
                lower.contains("429")
                    || lower.contains("rate limit")
                    || lower.contains("too many requests")
                    || lower.contains("timeout")
                    || lower.contains("temporar")
                    || lower.contains("unavailable")
                    || lower.contains("503")
            }
            ProviderError::ConfigError(_) => false,
        }
    }
}

#[async_trait]
impl LLMProvider for ReliableProvider {
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[serde_json::Value],
        options: &GenerationOptions,
    ) -> Result<GenerationResponse, ProviderError> {
        let mut backoff_ms = self.base_backoff_ms;
        let mut last_err: Option<ProviderError> = None;

        for attempt in 0..=self.max_retries {
            match self.inner.chat(messages, tools, options).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    if !Self::is_retryable(&err) || attempt == self.max_retries {
                        return Err(err);
                    }

                    tracing::warn!(
                        attempt = attempt + 1,
                        max_attempts = self.max_retries + 1,
                        backoff_ms,
                        "provider call failed; retrying"
                    );
                    last_err = Some(err);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms.saturating_mul(2)).min(2_000);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| ProviderError::ApiError("unknown provider failure".into())))
    }
}

