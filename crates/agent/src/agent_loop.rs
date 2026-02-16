use crate::context::ContextBuilder;
use crate::session::SessionManager;
use phoneclaw_core::audit::log_audit_internal;
use phoneclaw_core::bus::{Event, MessageBus};
use phoneclaw_core::config::AppConfig;
use phoneclaw_core::metrics::MetricsStore;
use phoneclaw_core::types::{Message, Role};
use phoneclaw_providers::{GenerationOptions, LLMProvider};
use phoneclaw_tools::registry::ToolRegistry;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Maximum tool-call loop iterations before the agent stops.
const MAX_ITERATIONS: usize = 10;
/// Number of LLM retry attempts on transient errors.
const LLM_RETRIES: usize = 3;
const PERSONA_BLOCK_START: &str = "<!-- phoneclaw:persona:start -->";
const PERSONA_BLOCK_END: &str = "<!-- phoneclaw:persona:end -->";

#[derive(Default)]
struct PersonaPreference {
    assistant_name: Option<String>,
    user_name: Option<String>,
}

pub struct AgentLoop {
    bus: Arc<MessageBus>,
    config: Arc<RwLock<AppConfig>>,
    provider: Arc<dyn LLMProvider>,
    tools: ToolRegistry,
    context_builder: ContextBuilder,
    sessions: SessionManager,
    metrics: Arc<MetricsStore>,
}

impl AgentLoop {
    pub fn new(
        bus: Arc<MessageBus>,
        config: Arc<RwLock<AppConfig>>,
        provider: Arc<dyn LLMProvider>,
        tools: ToolRegistry,
        context_builder: ContextBuilder,
        sessions: SessionManager,
        metrics: Arc<MetricsStore>,
    ) -> Self {
        Self {
            bus,
            config,
            provider,
            tools,
            context_builder,
            sessions,
            metrics,
        }
    }

    pub async fn run(&self) {
        let mut rx = self.bus.subscribe_inbound();

        info!("Agent loop started");

        loop {
            match rx.recv().await {
                Ok(msg) => {
                    self.process_message(msg).await;
                }
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    error!(
                        "Agent loop lagged by {} inbound messages (queue full)",
                        count
                    );
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Bus closed, stopping agent loop");
                    break;
                }
            }
        }
    }

    /// Call the LLM with retry + exponential backoff for transient failures.
    async fn call_llm_with_retry(
        &self,
        messages: &[Message],
        tool_defs: &[serde_json::Value],
        options: &GenerationOptions,
    ) -> Result<phoneclaw_providers::GenerationResponse, String> {
        let mut last_error = String::new();

        for attempt in 0..LLM_RETRIES {
            match self.provider.chat(messages, tool_defs, options).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_error = e.to_string();
                    if attempt < LLM_RETRIES - 1 {
                        let delay = std::time::Duration::from_millis(1000 * (1 << attempt));
                        warn!(
                            attempt = attempt + 1,
                            max = LLM_RETRIES,
                            delay_ms = delay.as_millis() as u64,
                            "LLM call failed, retrying: {}",
                            last_error
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error)
    }

    /// Send an error/warning message back to the user via the bus.
    fn send_error(&self, session_key: &str, text: &str) {
        let error_msg = Message::new("agent", session_key, Role::Assistant, text);
        if let Err(e) = self.bus.publish(Event::OutboundMessage(error_msg)) {
            error!("Failed to publish error outbound message: {}", e);
        }
    }

    async fn process_message(&self, msg: Message) {
        info!("Processing message: {}", msg.id);

        // 1. Update Session History
        self.sessions
            .add_message(&msg.session_key, msg.clone())
            .await;
        self.maybe_update_persona_from_chat(&msg.content).await;

        // 2. Build Context
        let history = self.sessions.get_history(&msg.session_key).await;
        let summary = self.sessions.get_summary(&msg.session_key).await;

        let history_len = history.len();
        let history_slice = if history_len > 0 {
            &history[0..history_len - 1]
        } else {
            &[]
        };

        let messages = self
            .context_builder
            .build(history_slice, summary.as_deref(), &msg.content);

        // 3. Prepare Tools (Filtered by Permissions)
        let allowed_tools = self.context_builder.get_allowed_tools();
        let tool_defs = self
            .tools
            .list_definitions_for_permissions(&allowed_tools)
            .await;

        if allowed_tools.is_empty() {
            warn!("No skills approved (or no tools allowed). Agent is running with 0 tools.");
        }

        // 4. Initial LLM Call
        let config_locked = self.config.read().await;
        let options = GenerationOptions {
            model: config_locked.agents.default.model.clone(),
            max_tokens: Some(config_locked.agents.default.max_tokens),
            temperature: Some(config_locked.agents.default.temperature),
        };
        drop(config_locked);

        let mut current_messages = messages.clone();
        let mut iteration = 0;

        while iteration < MAX_ITERATIONS {
            iteration += 1;

            let response = match self
                .call_llm_with_retry(&current_messages, &tool_defs, &options)
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    error!("LLM Provider error after {} retries: {}", LLM_RETRIES, e);
                    self.send_error(
                        &msg.session_key,
                        &format!(
                            "⚠️ I encountered an error communicating with the AI provider: {}",
                            e
                        ),
                    );
                    return;
                }
            };

            // Log token usage for observability/cost tracking
            if let Some(usage) = &response.usage {
                info!(
                    input_tokens = usage.input_tokens,
                    output_tokens = usage.output_tokens,
                    iteration = iteration,
                    session = %msg.session_key,
                    "LLM token usage"
                );

                self.metrics
                    .add_tokens(usage.input_tokens as u64, usage.output_tokens as u64);

                log_audit_internal(
                    "llm_completion",
                    &msg.session_key,
                    json!({
                        "model": options.model,
                        "input_tokens": usage.input_tokens,
                        "output_tokens": usage.output_tokens,
                        "iteration": iteration
                    }),
                );
            }

            let tool_calls = response.tool_calls.clone();
            let mut assistant_msg = Message::new(
                "agent",
                &msg.session_key,
                Role::Assistant,
                &response.content,
            );
            if !tool_calls.is_empty() {
                if let Ok(serialized) = serde_json::to_string(
                    &tool_calls
                        .iter()
                        .map(|tc| {
                            json!({
                                "id": tc.id.clone(),
                                "type": "function",
                                "function": {
                                    "name": tc.name.clone(),
                                    "arguments": tc.arguments.clone()
                                }
                            })
                        })
                        .collect::<Vec<_>>(),
                ) {
                    assistant_msg
                        .metadata
                        .insert("tool_calls_json".to_string(), serialized);
                }
            }
            // Add assistant response to context
            current_messages.push(assistant_msg);

            // If tool calls present
            if !tool_calls.is_empty() {
                for tool_call in tool_calls {
                    info!("Executing tool: {}", tool_call.name);

                    // Enforce Default Deny at execution time (Defense in Depth)
                    if !ToolRegistry::is_tool_allowed(&tool_call.name, &allowed_tools) {
                        warn!(
                            "Blocked tool execution: '{}' not in allowed list",
                            tool_call.name
                        );

                        log_audit_internal(
                            "security_violation",
                            &msg.session_key,
                            json!({
                                "type": "tool_blocked",
                                "tool": tool_call.name,
                                "reason": "default_deny"
                            }),
                        );

                        // feedback to agent
                        current_messages.push(Message::new(
                            "tool",
                            &msg.session_key,
                            Role::Tool,
                            &format!(
                                "Error: Tool '{}' is not authorized by any active skill.",
                                tool_call.name
                            ),
                        ));
                        continue;
                    }

                    let start = std::time::Instant::now();
                    self.metrics.inc_tool_calls();
                    let result = if let Some(tool) = self.tools.get(&tool_call.name).await {
                        // Permission guard
                        if !ToolRegistry::is_tool_allowed(&tool_call.name, &allowed_tools) {
                            format!(
                                "Permission denied: tool '{}' is not allowed",
                                tool_call.name
                            )
                        } else {
                            match serde_json::from_str(&tool_call.arguments) {
                                Ok(args) => match tool.execute(args).await {
                                    Ok(res) => res,
                                    Err(e) => format!("Error executing tool: {}", e),
                                },
                                Err(e) => format!("Error parsing arguments: {}", e),
                            }
                        }
                    } else {
                        format!("Tool not found: {}", tool_call.name)
                    };
                    let elapsed = start.elapsed();

                    // Track metrics
                    self.tools
                        .record_metrics(
                            &tool_call.name,
                            elapsed.as_millis() as u64,
                            !result.starts_with("Error")
                                && !result.starts_with("Permission denied")
                                && !result.starts_with("Tool not found"),
                        )
                        .await;

                    log_audit_internal(
                        "tool_execution",
                        &msg.session_key,
                        json!({
                            "tool": tool_call.name,
                            "args": tool_call.arguments, // string
                            "output_preview": if result.len() > 200 { &result[..200] } else { &result },
                            "duration_ms": elapsed.as_millis(),
                            "success": !result.starts_with("Error")
                        }),
                    );

                    let mut tool_msg = Message::new("agent", &msg.session_key, Role::Tool, &result);
                    tool_msg
                        .metadata
                        .insert("tool_call_id".to_string(), tool_call.id);
                    current_messages.push(tool_msg);
                }
                // Loop again to return results to LLM
                continue;
            }

            // Final Response
            let response_msg = Message::new(
                "agent",
                &msg.session_key,
                Role::Assistant,
                &response.content,
            );

            // Update Session
            self.sessions
                .add_message(&msg.session_key, response_msg.clone())
                .await;

            // Publish Outbound
            if let Err(e) = self.bus.publish(Event::OutboundMessage(response_msg)) {
                error!("Failed to publish outbound message: {}", e);
            }

            // Auto-summarize and trim history
            self.maybe_summarize_and_trim(&msg.session_key).await;
            return;
        }

        // Max iterations reached — notify user
        warn!(
            session = %msg.session_key,
            iterations = MAX_ITERATIONS,
            "Agent loop hit max iterations"
        );
        self.send_error(
            &msg.session_key,
            &format!(
                "⚠️ I reached the maximum number of processing steps ({}). My last response may be incomplete. Please try rephrasing your request.",
                MAX_ITERATIONS
            ),
        );
    }

    async fn maybe_summarize_and_trim(&self, session_key: &str) {
        let history = self.sessions.get_history(session_key).await;

        if self.sessions.should_summarize(session_key, history.len()) {
            info!(session = %session_key, "Auto-summarizing session history...");

            let system_prompt =
                "You are a helpful assistant. Summarize the conversation history concisely.";
            let user_prompt = format!(
                "Summarize the following conversation into a concise paragraph:\n\n{}",
                history
                    .iter()
                    .map(|m| format!("{:?}: {}", m.role, m.content))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            let messages = vec![
                Message::new("system", session_key, Role::System, system_prompt),
                Message::new("user", session_key, Role::User, &user_prompt),
            ];

            let config_locked = self.config.read().await;
            let options = GenerationOptions {
                model: config_locked.agents.default.model.clone(),
                max_tokens: Some(500),
                temperature: Some(0.3),
            };
            drop(config_locked);

            let tool_defs = vec![];

            match self
                .call_llm_with_retry(&messages, &tool_defs, &options)
                .await
            {
                Ok(resp) => {
                    let summary = resp.content;
                    self.sessions
                        .set_summary(session_key, summary.clone())
                        .await;
                    self.sessions.mark_summarized(session_key);

                    if let Some(usage) = &resp.usage {
                        self.metrics
                            .add_tokens(usage.input_tokens as u64, usage.output_tokens as u64);
                        info!(
                            session = %session_key,
                            input_tokens = usage.input_tokens,
                            output_tokens = usage.output_tokens,
                            "Auto-summary cost"
                        );
                    }

                    // Summarized! Now trim to keep last 10 messages
                    self.sessions.auto_trim_history(session_key, 10).await;
                }
                Err(e) => {
                    error!(session = %session_key, "Failed to auto-summarize: {}", e);
                }
            }
        }
    }

    async fn maybe_update_persona_from_chat(&self, content: &str) {
        let Some(preference) = Self::extract_persona_preference(content) else {
            return;
        };

        let workspace = {
            let config = self.config.read().await;
            config.workspace.clone()
        };

        if let Err(e) = Self::upsert_user_profile(&workspace, preference).await {
            warn!("Failed to update USER.md persona profile: {}", e);
        }
    }

    fn extract_persona_preference(content: &str) -> Option<PersonaPreference> {
        let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty() {
            return None;
        }
        let lower = normalized.to_lowercase();

        let user_name = Self::extract_named_value(
            &normalized,
            &lower,
            &[
                "hãy gọi tôi là",
                "hay goi toi la",
                "goi toi la",
                "call me",
                "you can call me",
                "address me as",
            ],
            &[
                " và tên của bạn là",
                " va ten cua ban la",
                " and your name is",
                " and call yourself",
                " and your name should be",
                ".",
                ",",
                ";",
                "!",
                "?",
            ],
        );

        let assistant_name = Self::extract_named_value(
            &normalized,
            &lower,
            &[
                "tên của bạn là",
                "ten cua ban la",
                "your name is",
                "call yourself",
                "you should call yourself",
                "hãy gọi bạn là",
                "hay goi ban la",
            ],
            &[".", ",", ";", "!", "?", " nhé", " nhe", " please"],
        );

        if user_name.is_none() && assistant_name.is_none() {
            None
        } else {
            Some(PersonaPreference {
                assistant_name,
                user_name,
            })
        }
    }

    fn extract_named_value(
        original: &str,
        lower: &str,
        markers: &[&str],
        stops: &[&str],
    ) -> Option<String> {
        for marker in markers {
            if let Some(start) = lower.find(marker) {
                let value_start = start + marker.len();
                if value_start >= original.len() || value_start >= lower.len() {
                    continue;
                }
                let mut end = lower.len();
                let tail_lower = &lower[value_start..];
                for stop in stops {
                    if let Some(idx) = tail_lower.find(stop) {
                        end = end.min(value_start + idx);
                    }
                }
                if end <= value_start || end > original.len() {
                    continue;
                }
                let raw = &original[value_start..end];
                if let Some(cleaned) = Self::sanitize_name(raw) {
                    return Some(cleaned);
                }
            }
        }
        None
    }

    fn sanitize_name(raw: &str) -> Option<String> {
        let cleaned = raw
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'' || c == '`')
            .trim_matches(|c: char| c == ':' || c == '-' || c == '=')
            .trim();

        if cleaned.is_empty() {
            return None;
        }
        let compact = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
        if compact.is_empty() || compact.len() > 80 {
            return None;
        }
        Some(compact)
    }

    async fn upsert_user_profile(
        workspace: &Path,
        preference: PersonaPreference,
    ) -> std::io::Result<()> {
        tokio::fs::create_dir_all(workspace).await?;
        let user_md = workspace.join("USER.md");
        let existing = tokio::fs::read_to_string(&user_md)
            .await
            .unwrap_or_default();

        let existing_assistant =
            Self::extract_existing_quoted_value(&existing, "- Refer to yourself as");
        let existing_user = Self::extract_existing_quoted_value(&existing, "- Address the user as");
        let existing_tone = Self::extract_existing_quoted_value(&existing, "- Maintain tone");

        let assistant_name = preference
            .assistant_name
            .or(existing_assistant)
            .unwrap_or_else(|| "PhoneClawbot".to_string());
        let user_name = preference
            .user_name
            .or(existing_user)
            .unwrap_or_else(|| "friend".to_string());
        let tone = existing_tone.unwrap_or_else(|| "friendly, concise".to_string());

        let block = format!(
            "## Preferred Addressing\n- Refer to yourself as \"{}\".\n- Address the user as \"{}\".\n- Maintain tone: \"{}\".\n- Apply this from the first reply unless the user asks to change.",
            assistant_name, user_name, tone
        );
        let merged = Self::replace_persona_block(&existing, &block);
        tokio::fs::write(user_md, format!("{}\n", merged.trim_end())).await
    }

    fn extract_existing_quoted_value(content: &str, prefix: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with(prefix) {
                continue;
            }
            let first_quote = trimmed.find('"')?;
            let rest = &trimmed[first_quote + 1..];
            let second_quote = rest.find('"')?;
            let value = &rest[..second_quote];
            if !value.trim().is_empty() {
                return Some(value.trim().to_string());
            }
        }
        None
    }

    fn replace_persona_block(existing: &str, block: &str) -> String {
        if let (Some(start), Some(end)) = (
            existing.find(PERSONA_BLOCK_START),
            existing.find(PERSONA_BLOCK_END),
        ) {
            if end > start {
                let prefix = existing[..start].trim_end();
                let suffix = existing[end + PERSONA_BLOCK_END.len()..].trim_start();
                let wrapped = format!("{}\n{}\n{}", PERSONA_BLOCK_START, block, PERSONA_BLOCK_END);
                return match (prefix.is_empty(), suffix.is_empty()) {
                    (true, true) => wrapped,
                    (true, false) => format!("{}\n\n{}", wrapped, suffix),
                    (false, true) => format!("{}\n\n{}", prefix, wrapped),
                    (false, false) => format!("{}\n\n{}\n\n{}", prefix, wrapped, suffix),
                };
            }
        }

        let wrapped = format!("{}\n{}\n{}", PERSONA_BLOCK_START, block, PERSONA_BLOCK_END);
        let base = existing.trim_end();
        if base.is_empty() {
            wrapped
        } else {
            format!("{}\n\n{}", base, wrapped)
        }
    }
}
