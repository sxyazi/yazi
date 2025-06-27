#!/usr/bin/env bash
set -euo pipefail

export ARTIFACT_NAME="yazi-$1"
export YAZI_GEN_COMPLETIONS=1

# Build the target
git config --global --add safe.directory "*"
cargo build --release --locked --target "$1"

# Use a consistent target directory
rm -rf target/release
mv "target/$1/release" target/release

# Package deb
if [[ "$ARTIFACT_NAME" == *-linux-* ]] && { [[ "$ARTIFACT_NAME" == *-aarch64-* ]] || [[ "$ARTIFACT_NAME" == *-x86_64-* ]]; }; then
	cargo install cargo-deb
	cargo deb -p yazi-packing --no-build -o "$ARTIFACT_NAME.deb"
fi

# Create the artifact
mkdir -p "$ARTIFACT_NAME/completions"
cp "target/release/ya" "$ARTIFACT_NAME"
cp "target/release/yazi" "$ARTIFACT_NAME"
cp yazi-cli/completions/* "$ARTIFACT_NAME/completions"
cp yazi-boot/completions/* "$ARTIFACT_NAME/completions"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &> /dev/null; then
	apt-get update && apt-get install -yq zip
fi
zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
