from pathlib import Path

import pytest

from infinite_porch.bootstrap_loader import BOOTSTRAP_CONTRACT_RELATIVE_PATH, load_bootstrap_context
from infinite_porch.errors import BootstrapContractNotFoundError, InvalidBootstrapContractError


def test_load_bootstrap_context_success(fixtures_root: Path) -> None:
    context = load_bootstrap_context(fixtures_root / "success")

    assert context.target == "porch"
    assert context.channel_name == "stable"
    assert context.candidate_id == "cand-001"
    assert [artifact.path for artifact in context.artifacts] == [
        "pantheon_data/consumers/porch/manifest.json",
        "pantheon_data/consumers/porch/private_graph.json",
        "pantheon_data/consumers/porch/private_chronicle.json",
        "pantheon_data/consumers/porch/optional_protocol_step.json",
    ]


def test_missing_contract_raises(tmp_path: Path) -> None:
    with pytest.raises(BootstrapContractNotFoundError):
        load_bootstrap_context(tmp_path)


def test_invalid_contract_raises(fixtures_root: Path) -> None:
    with pytest.raises(InvalidBootstrapContractError):
        load_bootstrap_context(fixtures_root / "invalid_contract")


def test_contract_path_constant() -> None:
    assert (
        str(BOOTSTRAP_CONTRACT_RELATIVE_PATH)
        == "pantheon_data/bootstrap/porch/latest_contract.json"
    )
