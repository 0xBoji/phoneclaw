# Summarize

Use when the user asks to summarize:
- latest news/topic
- long conversation/session context
- trend updates over time

## Workflow
1. If summary depends on chat history, use `sessions_history`.
2. If summary depends on current events, use `web_search`.
3. Present summary in 3 layers:
   - 1-line TL;DR
   - 3-5 key points
   - optional deeper section
4. Use `datetime_now` when topic is time-sensitive.
