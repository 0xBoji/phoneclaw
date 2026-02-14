package com.pocketclaw.app.wave

object ModelCatalog {
    val providers = arrayOf("openai", "google", "anthropic", "openrouter", "groq")

    val providerModels: Map<String, List<String>> = mapOf(
        "openai" to listOf(
            "gpt-4o-mini",
            "gpt-4o",
            "gpt-4.1-mini",
            "gpt-4.1",
            "gpt-4.1-nano",
            "gpt-4.5-preview",
            "o3-mini",
            "o3",
            "o4-mini",
            "gpt-4o-audio-preview"
        ),
        "google" to listOf(
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite",
            "gemini-1.5-flash",
            "gemini-1.5-pro",
            "gemini-2.5-flash-preview",
            "gemini-2.5-pro-preview",
            "gemini-embedding-001"
        ),
        "anthropic" to listOf(
            "claude-3-5-haiku-latest",
            "claude-3-5-sonnet-latest",
            "claude-3-7-sonnet-latest",
            "claude-opus-4-5",
            "claude-opus-4-6",
            "claude-sonnet-4-5",
            "claude-sonnet-4"
        ),
        "openrouter" to listOf(
            "openai/gpt-4o-mini",
            "openai/gpt-4.1",
            "openai/o3-mini",
            "anthropic/claude-3.5-sonnet",
            "anthropic/claude-3.7-sonnet",
            "anthropic/claude-opus-4.1",
            "google/gemini-2.0-flash-001",
            "google/gemini-1.5-pro",
            "meta-llama/llama-3.3-70b-instruct",
            "deepseek/deepseek-chat-v3-0324",
            "qwen/qwen-2.5-72b-instruct",
            "moonshotai/kimi-k2"
        ),
        "groq" to listOf(
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "mixtral-8x7b-32768",
            "gemma2-9b-it",
            "qwen-qwq-32b",
            "deepseek-r1-distill-llama-70b"
        )
    )
}
