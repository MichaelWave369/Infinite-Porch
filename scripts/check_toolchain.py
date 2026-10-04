#!/usr/bin/env python3
"""Reject drift between the tested MSRV, toolchain, CI and canonical README."""
import pathlib,re,tomllib
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
    cargo=tomllib.loads((ROOT/'Cargo.toml').read_text())
    version=cargo['workspace']['package']['rust-version']
    channel=tomllib.loads((ROOT/'rust-toolchain.toml').read_text())['toolchain']['channel']
    assert version==channel,(version,channel)
    assert f'dtolnay/rust-toolchain@{version}' in (ROOT/'.github/workflows/ci.yml').read_text()
    assert f'Required Rust: **{version}**' in (ROOT/'README.md').read_text()
    for p in ROOT.glob('crates/*/Cargo.toml'):
        assert tomllib.loads(p.read_text())['package']['rust-version']=={'workspace':True},str(p)
    print(f'Toolchain consistent: Rust {version}')
if __name__=='__main__':main()
