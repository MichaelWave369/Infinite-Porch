from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class BootstrapArtifactRef:
    name: str
    path: str
    required: bool = True
    order: int = 0


@dataclass(frozen=True)
class PorchBootstrapContext:
    bootstrap_contract_path: str
    target: str
    channel_name: str | None
    candidate_id: str | None
    artifacts: list[BootstrapArtifactRef]


@dataclass(frozen=True)
class PorchBootResult:
    bootstrap_contract_path: str
    target: str
    channel_name: str | None
    candidate_id: str | None
    loaded_artifacts: list[str]
    missing_required_artifacts: list[str]
    warnings: list[str]
    success: bool
    summary: str
