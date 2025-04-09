#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "Bumping version: $1"

TOML_FILES="$(git ls-files '*Cargo.toml')"
perl -pi -e "s/^version .*= .*\$/version = \"$1\"/" -- $TOML_FILES
perl -pi -e "s/^(yazi-[a-z]+)\\s*=\\s*{.*\$/\\1 = { path = \"..\/\\1\", version = \"$1\" }/" -- $TOML_FILES

ESLINT_USE_FLAT_CONFIG=true eslint -c ~/.config/rules/eslint/eslint.config.cjs --fix -- $TOML_FILES
