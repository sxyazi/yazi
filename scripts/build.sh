#!/bin/bash
set -euo pipefail

export ARTIFACT_NAME="yazi-$1"
export YAZI_GEN_COMPLETIONS=1

# Setup Rust toolchain
if [[ "$1" == *-musl ]]; then
	rustup target add "$1"
else
	rustup toolchain install stable --profile minimal --target "$1"
fi

# Build for the target
cargo build -p yazi-cli --release --locked --target "$1"
cargo build -p yazi-fm --release --locked --target "$1"

# Create the artifact
mkdir -p "$ARTIFACT_NAME/completions"
cp "target/$1/release/ya" "$ARTIFACT_NAME"
cp "target/$1/release/yazi" "$ARTIFACT_NAME"
cp yazi-cli/completions/* "$ARTIFACT_NAME/completions"
cp yazi-boot/completions/* "$ARTIFACT_NAME/completions"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &> /dev/null
then
	sudo apt-get update && sudo apt-get install -yq zip
fi
zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
