use pocketclaw_core::types::{Message, Role};
use pocketclaw_skills::SkillsLoader;
use std::path::PathBuf;

/// Maximum number of conversation history messages to include in context.
/// This prevents exceeding LLM token limits as conversations grow.
const MAX_HISTORY_MESSAGES: usize = 20;

pub struct ContextBuilder {
    workspace: PathBuf,
    skills_loader: SkillsLoader,
}

impl ContextBuilder {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            workspace: workspace.clone(),
            skills_loader: SkillsLoader::new(workspace),
        }
    }

    pub fn build(
        &self,
        history: &[Message],
        summary: Option<&str>,
        current_message: &str,
    ) -> Vec<Message> {
        let mut messages = Vec::new();

        // 1. System Prompt
        let system_prompt = self.build_system_prompt();
        messages.push(Message::new("system", "global", Role::System, &system_prompt));

        // 2. Summary (Long-term memory or compressed context)
        if let Some(s) = summary {
             messages.push(Message::new("system", "global", Role::System, &format!("Previous conversation summary: {}", s)));
        }

        // 3. Relevant Skills (load always-on skills)
        let skills = self.skills_loader.list_skills();
        for skill in skills {
            if skill.always && skill.available {
                messages.push(Message::new("system", "global", Role::System, &format!("Skill: {}\n{}", skill.name, skill.content)));
            }
        }

        // 4. Conversation History (sliding window — only last N messages)
        let history_window = if history.len() > MAX_HISTORY_MESSAGES {
            // Include a note that older messages were trimmed
            messages.push(Message::new(
                "system",
                "global",
                Role::System,
                &format!(
                    "[{} older messages omitted — see summary above for context]",
                    history.len() - MAX_HISTORY_MESSAGES
                ),
            ));
            &history[history.len() - MAX_HISTORY_MESSAGES..]
        } else {
            history
        };
        messages.extend_from_slice(history_window);

        // 5. Current Message
        messages.push(Message::new("cli", "current", Role::User, current_message));

        messages
    }

    fn build_system_prompt(&self) -> String {
        let mut prompt = String::from("You are PocketClaw, an intelligent AI assistant.\n");
        prompt.push_str("You must answer the user's request accurately and concisely.\n");
        prompt.push_str("If you need to perform actions, use the provided tools.\n");

        // Load workspace context files if they exist
        let context_files = ["AGENTS.md", "SOUL.md", "USER.md", "TOOLS.md", "IDENTITY.md"];
        for filename in &context_files {
            let path = self.workspace.join(filename);
            if let Ok(content) = std::fs::read_to_string(&path) {
                prompt.push_str(&format!("\n--- {} ---\n{}\n", filename, content));
            }
        }

        prompt
    }
}
