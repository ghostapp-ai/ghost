#!/usr/bin/env bash
# Updates version across all project files during semantic-release prepare step.
# Usage: bash scripts/update-versions.sh <version>
set -euo pipefail

VERSION="$1"

echo "Updating versions to $VERSION..."

# Update package.json
jq --arg v "$VERSION" '.version = $v' package.json > package.tmp.json
mv package.tmp.json package.json

# Update tauri.conf.json
jq --arg v "$VERSION" '.version = $v' src-tauri/tauri.conf.json > src-tauri/tauri.conf.tmp.json
mv src-tauri/tauri.conf.tmp.json src-tauri/tauri.conf.json

# Update Cargo.toml (first occurrence of version in [package] section)
sed -i "0,/^version = \".*\"/s//version = \"$VERSION\"/" src-tauri/Cargo.toml

echo "✓ All versions updated to $VERSION"
