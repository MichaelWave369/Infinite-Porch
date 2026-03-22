from __future__ import annotations

from pathlib import Path

from .artifact_loader import load_artifact_json
from .bootstrap_loader import load_bootstrap_context
from .errors import (
    ArtifactMalformedError,
    ArtifactMissingError,
    ArtifactUnreadableError,
    PorchError,
)
from .models import PorchBootResult


def run_bootstrap_proof(repo_root: Path) -> PorchBootResult:
    context = load_bootstrap_context(repo_root)
    loaded_artifacts: list[str] = []
    missing_required_artifacts: list[str] = []
    warnings: list[str] = []

    for artifact in context.artifacts:
        try:
            parsed = load_artifact_json(repo_root, artifact.path, required=artifact.required)
        except ArtifactMissingError:
            missing_required_artifacts.append(artifact.path)
            return PorchBootResult(
                bootstrap_contract_path=context.bootstrap_contract_path,
                target=context.target,
                channel_name=context.channel_name,
                candidate_id=context.candidate_id,
                loaded_artifacts=loaded_artifacts,
                missing_required_artifacts=missing_required_artifacts,
                warnings=warnings,
                success=False,
                summary=f"Bootstrap proof failed: missing required artifact {artifact.path}",
            )
        except (ArtifactMalformedError, ArtifactUnreadableError) as exc:
            return PorchBootResult(
                bootstrap_contract_path=context.bootstrap_contract_path,
                target=context.target,
                channel_name=context.channel_name,
                candidate_id=context.candidate_id,
                loaded_artifacts=loaded_artifacts,
                missing_required_artifacts=missing_required_artifacts,
                warnings=warnings,
                success=False,
                summary=f"Bootstrap proof failed: {exc}",
            )

        if parsed is None:
            warnings.append(f"Optional artifact missing: {artifact.path}")
            continue

        loaded_artifacts.append(artifact.path)

    return PorchBootResult(
        bootstrap_contract_path=context.bootstrap_contract_path,
        target=context.target,
        channel_name=context.channel_name,
        candidate_id=context.candidate_id,
        loaded_artifacts=loaded_artifacts,
        missing_required_artifacts=missing_required_artifacts,
        warnings=warnings,
        success=True,
        summary=(
            f"Bootstrap proof succeeded for target '{context.target}' "
            f"with {len(loaded_artifacts)} artifacts loaded."
        ),
    )


def run_bootstrap_proof_safe(repo_root: Path) -> PorchBootResult:
    try:
        return run_bootstrap_proof(repo_root)
    except PorchError as exc:
        return PorchBootResult(
            bootstrap_contract_path=str(
                (repo_root / "pantheon_data/bootstrap/porch/latest_contract.json").resolve()
            ),
            target="porch",
            channel_name=None,
            candidate_id=None,
            loaded_artifacts=[],
            missing_required_artifacts=[],
            warnings=[],
            success=False,
            summary=f"Bootstrap proof failed: {exc}",
        )
