use crate::anthropic::AnthropicProvider;
use crate::google::GoogleProvider;
use crate::openai::OpenAIProvider;
use crate::reliable::ReliableProvider;
use crate::LLMProvider;
use phoneclaw_core::config::AppConfig;
use std::sync::Arc;

/// Create the appropriate LLM provider based on the application config.
/// Checks providers in order: OpenAI → OpenRouter → Anthropic → Google.
pub fn create_provider(config: &AppConfig) -> anyhow::Result<Arc<dyn LLMProvider>> {
    let provider: Arc<dyn LLMProvider> = if let Some(openai_cfg) = &config.providers.openai {
        Arc::new(OpenAIProvider::new(
            openai_cfg.api_key.clone(),
            openai_cfg.api_base.clone(),
        ))
    } else if let Some(openrouter_cfg) = &config.providers.openrouter {
        Arc::new(OpenAIProvider::new(
            openrouter_cfg.api_key.clone(),
            openrouter_cfg.api_base.clone(),
        ))
    } else if let Some(anthropic_cfg) = &config.providers.anthropic {
        Arc::new(AnthropicProvider::new(
            anthropic_cfg.api_key.clone(),
        ))
    } else if let Some(google_cfg) = &config.providers.google {
        Arc::new(GoogleProvider::new(
            google_cfg.api_key.clone(),
            google_cfg.model.clone(),
        ))
    } else {
        anyhow::bail!("No LLM provider configured. Run 'phoneclaw onboard' to set one up.");
    };

    // Mobile-first reliability defaults: 2 retries, 250ms exponential backoff.
    Ok(Arc::new(ReliableProvider::new(provider, 2, 250)))
}
