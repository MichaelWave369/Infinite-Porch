from __future__ import annotations

import json
from pathlib import Path

from .errors import (
    BootstrapContractNotFoundError,
    InvalidBootstrapContractError,
    UnsafePathError,
)
from .models import BootstrapArtifactRef, PorchBootstrapContext

BOOTSTRAP_CONTRACT_RELATIVE_PATH = Path("pantheon_data/bootstrap/porch/latest_contract.json")


def _normalize_under_root(repo_root: Path, rel_path: str) -> Path:
    candidate = (repo_root / rel_path).resolve()
    root = repo_root.resolve()
    if not candidate.is_relative_to(root):
        raise UnsafePathError(f"Path escapes repository root: {rel_path}")
    return candidate


def load_bootstrap_context(repo_root: Path) -> PorchBootstrapContext:
    contract_path = (repo_root / BOOTSTRAP_CONTRACT_RELATIVE_PATH).resolve()
    if not contract_path.exists():
        raise BootstrapContractNotFoundError(
            f"Missing bootstrap contract: {BOOTSTRAP_CONTRACT_RELATIVE_PATH}"
        )

    try:
        payload = json.loads(contract_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise InvalidBootstrapContractError(
            f"Bootstrap contract is not valid JSON: {contract_path}"
        ) from exc

    if not isinstance(payload, dict):
        raise InvalidBootstrapContractError("Bootstrap contract must be a JSON object")

    target = str(payload.get("target", "porch"))
    channel_name = payload.get("channel_name")
    candidate_id = payload.get("candidate_id")
    raw_artifacts = payload.get("artifacts")

    if not isinstance(raw_artifacts, list) or not raw_artifacts:
        raise InvalidBootstrapContractError(
            "Bootstrap contract must include a non-empty artifacts list"
        )

    artifacts: list[BootstrapArtifactRef] = []
    for idx, raw in enumerate(raw_artifacts):
        if not isinstance(raw, dict):
            raise InvalidBootstrapContractError("Each artifact entry must be an object")
        name = str(raw.get("name") or f"artifact_{idx}")
        path = raw.get("path")
        if not isinstance(path, str) or not path:
            raise InvalidBootstrapContractError(f"Artifact {name} missing path")
        _normalize_under_root(repo_root, path)

        required = bool(raw.get("required", True))
        order_value = raw.get("order", idx)
        if not isinstance(order_value, int):
            raise InvalidBootstrapContractError(f"Artifact {name} has non-integer order")
        artifacts.append(
            BootstrapArtifactRef(name=name, path=path, required=required, order=order_value)
        )

    artifacts_sorted = sorted(artifacts, key=lambda item: item.order)

    return PorchBootstrapContext(
        bootstrap_contract_path=str(contract_path),
        target=target,
        channel_name=channel_name if isinstance(channel_name, str) else None,
        candidate_id=candidate_id if isinstance(candidate_id, str) else None,
        artifacts=artifacts_sorted,
    )
