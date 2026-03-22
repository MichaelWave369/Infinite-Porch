from __future__ import annotations

import argparse
import json
from dataclasses import asdict
from pathlib import Path

from .proof_service import run_bootstrap_proof_safe


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="porch")
    subcommands = parser.add_subparsers(dest="command", required=True)

    check = subcommands.add_parser("bootstrap-check", help="Run deterministic bootstrap proof")
    check.add_argument(
        "--repo-root",
        type=Path,
        default=Path.cwd(),
        help="Repository root containing pantheon_data",
    )

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.command == "bootstrap-check":
        result = run_bootstrap_proof_safe(args.repo_root)
        print(result.summary)
        print(json.dumps(asdict(result), indent=2, sort_keys=True))
        return 0 if result.success else 1

    parser.error(f"Unsupported command: {args.command}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
