#!/bin/sh
set -eu
PORCH_SOURCE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
PORCH_INSTALL_DIR=${1:-"$HOME/.local/share/infinite-porch/0.1.1"}
PORCH_STATE_DIR=${2:-"${XDG_STATE_HOME:-"$HOME/.local/state"}/infinite-porch"}
[ -x "$PORCH_SOURCE_DIR/bin/porch-node" ] || { echo 'Extract a native portable package first.' >&2; exit 1; }
mkdir -p "$PORCH_INSTALL_DIR" "$PORCH_STATE_DIR"
chmod 700 "$PORCH_STATE_DIR"
cp -R "$PORCH_SOURCE_DIR/bin" "$PORCH_SOURCE_DIR/ui" "$PORCH_INSTALL_DIR/"
printf 'Installed: %s\nState: %s\n' "$PORCH_INSTALL_DIR" "$PORCH_STATE_DIR"
printf 'Initialize: "%s/bin/porch" --data "%s" init --alias MIKEY-PC\n' "$PORCH_INSTALL_DIR" "$PORCH_STATE_DIR"
printf 'Launch: "%s/bin/porch-node" --data "%s" --ui "%s/ui"\n' "$PORCH_INSTALL_DIR" "$PORCH_STATE_DIR" "$PORCH_INSTALL_DIR"
