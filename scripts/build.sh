#!/bin/bash
set -euo pipefail

export ARTIFACT_NAME="yazi-$1"
export YAZI_GEN_COMPLETIONS=1
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc

# Setup Rust toolchain
rustup toolchain install stable --profile minimal
rustup target add "$1"

# Build for the target
cargo build --release --locked --target "$1"

# Create the artifact
mkdir "$ARTIFACT_NAME"
cp "target/$1/release/ya" "$ARTIFACT_NAME"
cp "target/$1/release/yazi" "$ARTIFACT_NAME"
cp -r yazi-boot/completions "$ARTIFACT_NAME"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &> /dev/null
then
	sudo apt-get update && sudo apt-get install -yq zip
fi
zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
