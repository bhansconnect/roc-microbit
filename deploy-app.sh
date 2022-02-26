#!/bin/sh
set -e

# Build platform.
./build-app.sh $1

# Deploy platform.
(cd platform && cargo run --release)
