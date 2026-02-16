# Weather

Use this skill when the user asks for:
- current weather
- forecast for today/tomorrow/weekend
- weather comparison between cities

## Workflow
1. Use `datetime_now` to anchor the response time.
2. Use `web_search` with focused query:
   - `weather <city> now`
   - `<city> 7 day weather forecast`
3. Return concise weather summary first, then details (temp, rain chance, wind) if user asks.

## Response style
- Always include location and date/time basis.
- If confidence is low, say so and ask for clarification (city/country).
