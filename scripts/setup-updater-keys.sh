#!/usr/bin/env bash
# Generate signing keys for Tauri auto-updater.
#
# Usage:
#   bash scripts/setup-updater-keys.sh
#
# This generates a keypair for signing app updates:
#   - Private key: ~/.tauri/ghost.key (KEEP SECRET — add to CI secrets)
#   - Public key:  ~/.tauri/ghost.key.pub (add to tauri.conf.json)
#
# After running this script:
# 1. Copy the contents of ~/.tauri/ghost.key.pub into
#    src-tauri/tauri.conf.json → plugins.updater.pubkey
# 2. Add these secrets to your GitHub repository:
#    - TAURI_SIGNING_PRIVATE_KEY: contents of ~/.tauri/ghost.key
#    - TAURI_SIGNING_PRIVATE_KEY_PASSWORD: the password you set

set -euo pipefail

KEY_DIR="$HOME/.tauri"
KEY_FILE="$KEY_DIR/ghost.key"

mkdir -p "$KEY_DIR"

if [ -f "$KEY_FILE" ]; then
  echo "Warning: Key already exists at $KEY_FILE"
  echo "Delete it first if you want to regenerate."
  exit 1
fi

echo "Generating Tauri updater signing keys..."
echo ""

# Use bun if available, otherwise npx
if command -v bun &>/dev/null; then
  bun run tauri signer generate -- -w "$KEY_FILE"
elif command -v npx &>/dev/null; then
  npx tauri signer generate -- -w "$KEY_FILE"
else
  cargo tauri signer generate -w "$KEY_FILE"
fi

echo ""
echo "Keys generated successfully!"
echo ""
echo "Next steps:"
echo ""
echo "1. Copy public key to tauri.conf.json:"
echo "   cat $KEY_FILE.pub"
echo ""
echo "2. Add secrets to GitHub repository (Settings > Secrets > Actions):"
echo "   - TAURI_SIGNING_PRIVATE_KEY = contents of $KEY_FILE"
echo "   - TAURI_SIGNING_PRIVATE_KEY_PASSWORD = the password you just set"
echo ""
echo "NEVER commit the private key to the repository!"
