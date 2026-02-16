# Healthcheck

Use this for runtime diagnostics.

## Workflow
1. `datetime_now`
2. `channel_health`
3. `metrics_snapshot`
4. Return:
   - current status
   - impacted channels
   - top error signals
   - recommended next action

## Output format
- Status: OK/WARN/FAIL
- Key metrics snapshot
- Immediate actions (1-3 bullets)
