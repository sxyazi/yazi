#!/usr/bin/env bash
set -euo pipefail

export ARTIFACT_NAME="yazi-$1"
export YAZI_GEN_COMPLETIONS=1

# Build for the target
git config --global --add safe.directory "*"
cargo build --release --locked --target "$1"

# Create the artifact
mkdir -p "$ARTIFACT_NAME/completions"
cp "target/$1/release/ya" "$ARTIFACT_NAME"
cp "target/$1/release/yazi" "$ARTIFACT_NAME"
cp yazi-cli/completions/* "$ARTIFACT_NAME/completions"
cp yazi-boot/completions/* "$ARTIFACT_NAME/completions"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &> /dev/null; then
	apt-get update && apt-get install -yq zip
fi
zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"

# build deb package (see https://crates.io/crates/cargo-deb)
## run it after build
cargo deb -p yazi-fm --no-build
