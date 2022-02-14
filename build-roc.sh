#!/bin/sh
nix-shell \
	--command 'cd roc && cargo build --release --features target-arm' \
	./roc/shell.nix
