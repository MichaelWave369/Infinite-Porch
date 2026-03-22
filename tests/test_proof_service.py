from pathlib import Path

from infinite_porch.proof_service import run_bootstrap_proof_safe


def test_proof_success(fixtures_root: Path) -> None:
    result = run_bootstrap_proof_safe(fixtures_root / "success")

    assert result.success is True
    assert result.missing_required_artifacts == []
    assert result.loaded_artifacts == [
        "pantheon_data/consumers/porch/manifest.json",
        "pantheon_data/consumers/porch/private_graph.json",
        "pantheon_data/consumers/porch/private_chronicle.json",
    ]
    assert result.warnings == [
        "Optional artifact missing: pantheon_data/consumers/porch/optional_protocol_step.json"
    ]


def test_proof_missing_required(fixtures_root: Path) -> None:
    result = run_bootstrap_proof_safe(fixtures_root / "missing_required")

    assert result.success is False
    assert result.missing_required_artifacts == ["pantheon_data/consumers/porch/private_graph.json"]
    assert "missing required artifact" in result.summary


def test_proof_malformed_artifact(fixtures_root: Path) -> None:
    result = run_bootstrap_proof_safe(fixtures_root / "malformed")

    assert result.success is False
    assert result.missing_required_artifacts == []
    assert "Malformed JSON artifact" in result.summary
