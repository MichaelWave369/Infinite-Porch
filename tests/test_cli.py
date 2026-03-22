from pathlib import Path

from infinite_porch.cli import main


def test_cli_success(fixtures_root: Path, capsys) -> None:
    code = main(["bootstrap-check", "--repo-root", str(fixtures_root / "success")])
    captured = capsys.readouterr()

    assert code == 0
    assert "Bootstrap proof succeeded" in captured.out
    assert '"success": true' in captured.out


def test_cli_failure(fixtures_root: Path, capsys) -> None:
    code = main(["bootstrap-check", "--repo-root", str(fixtures_root / "missing_required")])
    captured = capsys.readouterr()

    assert code == 1
    assert "Bootstrap proof failed" in captured.out
    assert '"success": false' in captured.out
