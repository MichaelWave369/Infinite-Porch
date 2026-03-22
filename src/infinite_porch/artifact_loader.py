from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .errors import (
    ArtifactMalformedError,
    ArtifactMissingError,
    ArtifactUnreadableError,
    UnsafePathError,
)


def resolve_artifact_path(repo_root: Path, relative_path: str) -> Path:
    artifact_path = (repo_root / relative_path).resolve()
    root = repo_root.resolve()
    if not artifact_path.is_relative_to(root):
        raise UnsafePathError(f"Artifact path escapes repository root: {relative_path}")
    return artifact_path


def load_artifact_json(
    repo_root: Path, relative_path: str, *, required: bool
) -> dict[str, Any] | None:
    artifact_path = resolve_artifact_path(repo_root, relative_path)

    if not artifact_path.exists():
        if required:
            raise ArtifactMissingError(f"Missing required artifact: {relative_path}")
        return None

    if not artifact_path.is_file():
        raise ArtifactUnreadableError(f"Unreadable artifact path (not a file): {relative_path}")

    try:
        raw = artifact_path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ArtifactUnreadableError(f"Unable to read artifact: {relative_path}") from exc

    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ArtifactMalformedError(f"Malformed JSON artifact: {relative_path}") from exc

    if not isinstance(parsed, dict):
        raise ArtifactMalformedError(f"Artifact JSON must be an object: {relative_path}")

    return parsed
