#!/bin/sh
set -e

nix-shell \
	--command 'cd roc && cargo build --release --features target-arm' \
	./roc/shell.nix
