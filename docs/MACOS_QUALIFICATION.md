# macOS source qualification — UNVERIFIED

No native macOS executable, signed .app, notarization or physical macOS test is
claimed. The configured CI matrix does not constitute an executed macOS build.

Install Xcode Command Line Tools (`xcode-select --install`), Git, Rust 1.89.0,
Node 22.12+ and Python 3.12+. Restore the git bundle and build locally:

```sh
git clone InfinitePorch.gitbundle InfinitePorch-git
cd InfinitePorch-git
rustup toolchain install 1.89.0 --component clippy,rustfmt
cargo build --release --workspace --locked
npm ci
npm run build
python3 scripts/package.py --skip-build
```

Use your generated macos archive, not the Linux binary. Keep state under
`$HOME/Library/Application Support/InfinitePorch/state` with mode 0700. Initialize
with `target/release/porch --data "$HOME/Library/Application Support/InfinitePorch/state" init --alias MAC-A`,
then start `target/release/porch-node` with the same --data and
`--ui apps/desktop/dist`. Set the same --data on every control command.

Permit incoming peer connections only on your trusted network if the macOS
firewall prompts. Keep API 7331 and Ollama 11434 loopback. mDNS may be blocked on
guest Wi-Fi; use a signed recipient-bound invitation and pinned literal LAN IP.
Install/start local Ollama and scan before sharing. Follow PHYSICAL_QUALIFICATION
and the Linux command sequence with the macOS executable paths and state path.

The desktop artifact is the browser UI served by the node. Gatekeeper and
quarantine behavior, native paths, permissions, packet transport and Ollama
integration must be tested on an actual Mac. Do not treat unsigned local builds
as publicly trusted. No instruction here disables system-wide security controls.
