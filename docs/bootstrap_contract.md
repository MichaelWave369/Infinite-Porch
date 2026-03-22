# Porch Bootstrap Contract

Infinite Porch consumes a local Pantheon Porch bootstrap contract from:

- `pantheon_data/bootstrap/porch/latest_contract.json`

The contract is expected to include:
- target metadata (`target`, `channel_name`, `candidate_id`)
- ordered artifact references with `path`, `required`, and optional `order`

Infinite Porch parses these references, normalizes each path beneath repo root, and loads artifacts in ascending order.

A successful proof means:
- the bootstrap contract exists and parses as JSON
- all required artifacts exist and parse cleanly
- optional missing artifacts are recorded as warnings

Common failures:
- missing contract file
- missing required artifact
- malformed JSON in contract or artifact
- unreadable paths or path traversal outside repository root
