#!/usr/bin/env bash
set -euo pipefail

git config --global --add safe.directory "*"

if ! command -v zip &> /dev/null; then
	apt-get update && apt-get install -yq zip
fi

cargo xtask dist --target "$1"
