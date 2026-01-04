#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "Bumping version: $1"

TOML_FILES="$(git ls-files '*Cargo.toml')"
perl -pi -e 's/^version .*= .*$/version = "'"$1"'"/' -- $TOML_FILES
perl -pi -e 's/^(yazi-[a-z]+)\s*=\s*{.*$/\1 = { path = "..\/\1", version = "'"$1"'" }/' -- $TOML_FILES

# Insert "## [v$1]" after "## [Unreleased]"
perl -0777 -pe "s/^(## \[Unreleased\]\s*)/\\1## [v$1]\n\n/m" -i CHANGELOG.md
# Determine previous version and append compare link
prev_ver=$(grep -oE '^\[v[0-9][^]]+\]' CHANGELOG.md | tail -n1 | tr -d '[]')
link="\[v$1\]: https://github.com/sxyazi/yazi/compare/$prev_ver...v$1"
perl -pi -e 's{(\['"$prev_ver"'\]:[^\n]+\n)}{$1'"$link"'\n}s' CHANGELOG.md

ESLINT_USE_FLAT_CONFIG=true eslint -c ~/.config/rules/eslint/eslint.config.cjs --fix -- $TOML_FILES
