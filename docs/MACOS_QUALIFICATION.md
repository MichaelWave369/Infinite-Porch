# macOS qualification for 0.1.2

The delivery report identifies the actual native runner, CPU architecture,
tested commit and portable ZIP. A configured workflow alone proves no result.
Native hosted software tests do not qualify physical LAN, household WAN-off,
physical remote Ollama, Gatekeeper behavior on another Mac or independent
security review. This candidate is **NOT PRODUCTION QUALIFIED**.

Use the matching native macOS ZIP under the outer delivery's `packages/`.
Compare its SHA-256 with the delivery report before extracting. The manifest
contains per-file hashes; binaries are unsigned and not notarized. Only the
architecture recorded in the report is validated; no universal-binary claim is
made. The desktop is the browser UI served by the native node.

From the extracted native package, launch `./start.sh` for foreground operation
with state under that user-owned package directory. To use a separate persistent
per-user installation and state directory:

```sh
PORCH_INSTALL_DIR="$HOME/Library/Application Support/InfinitePorch/0.1.2"
PORCH_STATE_DIR="$HOME/Library/Application Support/InfinitePorch/state"
sh scripts/platform/install-user.sh "$PORCH_INSTALL_DIR" "$PORCH_STATE_DIR"
"$PORCH_INSTALL_DIR/bin/porch" --data "$PORCH_STATE_DIR" init --alias MAC-A
"$PORCH_INSTALL_DIR/bin/porch-node" --data "$PORCH_STATE_DIR" --ui "$PORCH_INSTALL_DIR/ui"
```

The helper creates private state with mode 0700. Retain the same state path for
all later control commands. Open `http://127.0.0.1:7331` and use only that node's
local `api.token`. Keep keys, tokens and databases on the original machine.
Stop the foreground node with Ctrl+C. No OS service is installed.

Review any macOS security prompt through the normal system UI. Do not disable
Gatekeeper or other system-wide controls. Permit peer connections only on a
trusted network if the firewall prompts. Keep control API 7331 and Ollama
11434 loopback. Guest Wi-Fi may block peer discovery; use a recipient-bound
signed invitation and a verified full host fingerprint for manual pairing.

For source work, the outer `repository-history.bundle` preserves the candidate
branch and baseline tags. Restore it from the **outer delivery directory**:

```sh
git clone --branch feat/porch-v0.1.2-native-qualification repository-history.bundle InfinitePorch-git
cd InfinitePorch-git
rustup toolchain install 1.89.0 --component clippy,rustfmt
cargo build --release --workspace --locked
npm ci
npm run build
python3 scripts/package.py --skip-build
```

Source builds require Xcode Command Line Tools, Git, Rust 1.89.0, Node 22.12+
and Python 3.12+. Build and execute a native package for the actual machine;
a Linux executable does not establish macOS qualification. Physical operation
requires independently exported participant evidence and correlation. GPU use,
sustained load, Intel macOS and other unrecorded environments remain UNVERIFIED.
