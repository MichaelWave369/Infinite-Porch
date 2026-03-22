# Local Development

## Setup

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e .[dev]
```

## Run checks

```bash
ruff check .
pytest
```

## Run porch bootstrap proof

```bash
porch bootstrap-check
```

Use a custom root when your `pantheon_data` directory lives elsewhere:

```bash
porch bootstrap-check --repo-root /path/to/worktree
```

The command emits:
- human-readable summary
- machine-readable JSON result

Exit codes:
- `0` success
- `1` failure
