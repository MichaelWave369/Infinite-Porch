# Real Ollama qualification

Provider tests in cargo test use owned HTTP fixtures without model weights.
Mock CI remains explicit. Live inference is an independent environment gate.

The adapter uses literal loopback HTTP, no proxy, no redirects, bounded response
bytes, at most 30 seconds and 4096 requested output tokens. It checks /api/tags
before and after inference and binds the reported model digest. A changed model
fails closed until the operator re-shares it; existing model grants are revoked
when sharing a changed version/provider. Cancellation suppresses output but is
not proof that Ollama stopped consuming hardware immediately.

```sh
porch models scan
porch models refresh
porch share model EXACT_INSTALLED_MODEL
porch models verify EXACT_INSTALLED_MODEL
python3 scripts/qualify_ollama.py --data state --model EXACT_INSTALLED_MODEL --out live-ollama.json
```

On Windows use the platform guide's executable and state paths. The script's
--cli selects a packaged executable. Inventory is not automatically configured
or shared. Observe five independent states: advertised; last verified;
currently reachable with an observation timestamp/30-second freshness; granted
invocation permission; last invocation successful. /api/ps provides loaded state,
which is separate from installed state: an unloaded model can load on invocation.
The background scan runs every 15 seconds; execution still rechecks immediately.

| Scenario | Procedure | Expected evidence |
| --- | --- | --- |
| Provider present | Start local Ollama, scan | reachable observation and inventory |
| Provider absent/unreachable | Run scan while Ollama is unavailable | explicit false, no mock substitution |
| Model present | Select a current scan entry, verify | real provider receipt or bounded failure |
| Model absent | Use an exact nonexistent name | refusal; no provider inference |
| Model unloaded | Unload through Ollama's supported interface, scan /ps, then verify | installed true, loaded false before, actual invocation result separately |
| List/version changes | Remove/change only a disposable model and scan | inventory changes; digest mismatch fails closed and requires explicit re-share |
| Timeout | Use a disposable test model that exceeds the operator deadline | refusal, null output, failed last invocation state |
| Provider process death | On your disposable test setup, stop your own Ollama during an authorized public probe | bounded refusal/transport deadline; no substitute output |

Never stop unrelated processes automatically. Dynamic contract cases are covered
by `cargo test -p porch-node --test provider_runtime` and process-failures.py.
Their PASS results do not count as live-model evidence. The opt-in live suite
records absent environments as SKIPPED_ENVIRONMENT and operator-controlled
failure cases as UNVERIFIED until executed. Use public prompts for qualification.

Ollama's reported digest is provider evidence; 0.1.1 does not measure GPU isolation,
prove model weights cryptographically, or reserve hardware. Official API sources:
https://docs.ollama.com/api/tags, https://docs.ollama.com/api/ps,
https://docs.ollama.com/api/generate.
