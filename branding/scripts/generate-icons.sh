#!/usr/bin/env bash
# =============================================================================
# Ghost — Brand Asset Generator
# =============================================================================
# Generates all icons, PNGs, and platform-specific assets from master SVGs.
#
# Requirements:
#   - resvg (cargo install resvg)
#   - convert (ImageMagick — sudo apt install imagemagick)
#
# Usage:
#   ./branding/scripts/generate-icons.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
BRAND_DIR="$ROOT_DIR/branding"
SVG_DIR="$BRAND_DIR/svg"
PNG_DIR="$BRAND_DIR/png"
ICONS_DIR="$BRAND_DIR/icons"
SOCIAL_DIR="$BRAND_DIR/social"
TAURI_ICONS_DIR="$ROOT_DIR/src-tauri/icons"
PUBLIC_DIR="$ROOT_DIR/public"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
echo "  ╔═══════════════════════════════════════════╗"
echo "  ║     👻 Ghost Brand Asset Generator        ║"
echo "  ╚═══════════════════════════════════════════╝"
echo -e "${NC}"

# ── Check dependencies ───────────────────────────────────────────────────────
check_deps() {
    local missing=()
    command -v resvg >/dev/null 2>&1 || missing+=("resvg (cargo install resvg)")
    command -v convert >/dev/null 2>&1 || missing+=("convert (sudo apt install imagemagick)")

    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}Missing dependencies:${NC}"
        for dep in "${missing[@]}"; do
            echo -e "  ${YELLOW}→ $dep${NC}"
        done
        exit 1
    fi
    echo -e "${GREEN}✓ All dependencies found${NC}"
}

# ── Helper: SVG to PNG ──────────────────────────────────────────────────────
svg_to_png() {
    local svg="$1"
    local png="$2"
    local size="$3"
    resvg --width "$size" --height "$size" "$svg" "$png" 2>/dev/null
    echo -e "  ${GREEN}✓${NC} $(basename "$png") (${size}x${size})"
}

# ── Helper: Generate ICO from PNGs ──────────────────────────────────────────
generate_ico() {
    local output="$1"
    shift
    local inputs=("$@")
    convert "${inputs[@]}" "$output" 2>/dev/null
    echo -e "  ${GREEN}✓${NC} $(basename "$output")"
}

# ── Helper: Generate ICNS from PNG ──────────────────────────────────────────
generate_icns() {
    local src_png="$1"
    local output="$2"

    # ICNS generation requires png2icns or iconutil (macOS)
    # We'll use ImageMagick to create a basic ICNS-compatible set
    if command -v png2icns >/dev/null 2>&1; then
        local tmpdir
        tmpdir=$(mktemp -d)
        for size in 16 32 128 256 512; do
            resvg --width "$size" --height "$size" "$SVG_DIR/ghost-icon.svg" "$tmpdir/icon_${size}x${size}.png" 2>/dev/null
        done
        png2icns "$output" "$tmpdir"/icon_*.png 2>/dev/null
        rm -rf "$tmpdir"
        echo -e "  ${GREEN}✓${NC} $(basename "$output") (via png2icns)"
    else
        # Fallback: copy a high-res PNG and note that ICNS should be generated on macOS
        cp "$src_png" "${output%.icns}.png"
        echo -e "  ${YELLOW}⚠${NC} $(basename "$output") — png2icns not found, using PNG fallback"
        echo -e "    ${YELLOW}→ Generate proper ICNS on macOS with: iconutil -c icns icon.iconset${NC}"
    fi
}

# =============================================================================
# MAIN GENERATION
# =============================================================================

check_deps

# Create output dirs
mkdir -p "$PNG_DIR" "$ICONS_DIR" "$SOCIAL_DIR" "$TAURI_ICONS_DIR" "$PUBLIC_DIR"

# ─── 1. PNG EXPORTS (all sizes) ─────────────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Generating PNG exports ━━━${NC}"

# App icon - standard sizes
for size in 16 24 32 48 64 128 256 512 1024; do
    svg_to_png "$SVG_DIR/ghost-icon.svg" "$PNG_DIR/ghost-icon-${size}.png" "$size"
done

# Rounded app icon
for size in 128 256 512 1024; do
    svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$PNG_DIR/ghost-icon-rounded-${size}.png" "$size"
done

# Circle app icon
for size in 128 256 512; do
    svg_to_png "$SVG_DIR/ghost-icon-circle.svg" "$PNG_DIR/ghost-icon-circle-${size}.png" "$size"
done

# Monochrome variants
for size in 128 256 512; do
    svg_to_png "$SVG_DIR/ghost-mono-white.svg" "$PNG_DIR/ghost-mono-white-${size}.png" "$size"
    svg_to_png "$SVG_DIR/ghost-mono-dark.svg" "$PNG_DIR/ghost-mono-dark-${size}.png" "$size"
done

# Tray icon
for size in 16 22 24 32 48; do
    svg_to_png "$SVG_DIR/ghost-tray-icon.svg" "$PNG_DIR/ghost-tray-${size}.png" "$size"
done

# ─── 2. TAURI ICONS (required for build) ────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Generating Tauri icons ━━━${NC}"

# Tauri v2 required icons:
# - 32x32.png
# - 128x128.png
# - 128x128@2x.png (actually 256x256)
# - icon.ico (Windows)
# - icon.icns (macOS)
# - icon.png (fallback, 512x512 or 1024x1024)

svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/32x32.png" 32
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/128x128.png" 128
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/128x128@2x.png" 256
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/icon.png" 512

# Windows Store logos (UWP)
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/StoreLogo.png" 50
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square30x30Logo.png" 30
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square44x44Logo.png" 44
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square71x71Logo.png" 71
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square89x89Logo.png" 89
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square107x107Logo.png" 107
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square142x142Logo.png" 142
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square150x150Logo.png" 150
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square284x284Logo.png" 284
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$TAURI_ICONS_DIR/Square310x310Logo.png" 310

# ─── 3. ICO FILE (Windows) ───────────────────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Generating Windows ICO ━━━${NC}"

# Create temp PNGs for ICO (needs specific sizes)
ICO_TMP=$(mktemp -d)
for size in 16 24 32 48 64 128 256; do
    resvg --width "$size" --height "$size" "$SVG_DIR/ghost-icon-rounded.svg" "$ICO_TMP/icon-${size}.png" 2>/dev/null
done

generate_ico "$TAURI_ICONS_DIR/icon.ico" \
    "$ICO_TMP/icon-16.png" \
    "$ICO_TMP/icon-24.png" \
    "$ICO_TMP/icon-32.png" \
    "$ICO_TMP/icon-48.png" \
    "$ICO_TMP/icon-64.png" \
    "$ICO_TMP/icon-128.png" \
    "$ICO_TMP/icon-256.png"

rm -rf "$ICO_TMP"

# ─── 4. ICNS FILE (macOS) ────────────────────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Generating macOS ICNS ━━━${NC}"

generate_icns "$PNG_DIR/ghost-icon-512.png" "$TAURI_ICONS_DIR/icon.icns"

# ─── 5. FAVICONS (web) ──────────────────────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Generating web favicons ━━━${NC}"

# Copy SVG favicon
cp "$SVG_DIR/ghost-favicon.svg" "$PUBLIC_DIR/favicon.svg"
echo -e "  ${GREEN}✓${NC} favicon.svg"

# PNG favicons
svg_to_png "$SVG_DIR/ghost-favicon.svg" "$PUBLIC_DIR/favicon-16x16.png" 16
svg_to_png "$SVG_DIR/ghost-favicon.svg" "$PUBLIC_DIR/favicon-32x32.png" 32

# ICO favicon
ICO_FAV_TMP=$(mktemp -d)
resvg --width 16 --height 16 "$SVG_DIR/ghost-favicon.svg" "$ICO_FAV_TMP/fav-16.png" 2>/dev/null
resvg --width 32 --height 32 "$SVG_DIR/ghost-favicon.svg" "$ICO_FAV_TMP/fav-32.png" 2>/dev/null
resvg --width 48 --height 48 "$SVG_DIR/ghost-favicon.svg" "$ICO_FAV_TMP/fav-48.png" 2>/dev/null
generate_ico "$PUBLIC_DIR/favicon.ico" \
    "$ICO_FAV_TMP/fav-16.png" \
    "$ICO_FAV_TMP/fav-32.png" \
    "$ICO_FAV_TMP/fav-48.png"
rm -rf "$ICO_FAV_TMP"

# Apple touch icon
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$PUBLIC_DIR/apple-touch-icon.png" 180
echo -e "  ${GREEN}✓${NC} apple-touch-icon.png (180x180)"

# Android/Chrome icons
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$PUBLIC_DIR/icon-192.png" 192
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$PUBLIC_DIR/icon-512.png" 512

# ─── 6. SOCIAL MEDIA ASSETS ─────────────────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Generating social media assets ━━━${NC}"

# GitHub profile/org avatar (recommended: 500x500)
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$SOCIAL_DIR/github-avatar.png" 500

# Open Graph image base (1200x630 is standard for OG)
# We'll generate a centered icon for now — a proper OG image needs a background + text
svg_to_png "$SVG_DIR/ghost-icon-rounded.svg" "$SOCIAL_DIR/og-icon.png" 630

echo -e "  ${YELLOW}ℹ${NC}  For full OG images (1200x630), create a design in Figma/Canva with the ghost icon + text"

# ─── 7. COPY UPDATED LOGO TO PUBLIC ─────────────────────────────────────────
echo ""
echo -e "${BLUE}━━━ Updating public assets ━━━${NC}"

cp "$SVG_DIR/ghost-icon.svg" "$PUBLIC_DIR/ghost-logo.svg"
echo -e "  ${GREEN}✓${NC} ghost-logo.svg (updated)"

# ─── Summary ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All brand assets generated successfully!${NC}"
echo ""
echo -e "  📁 SVG masters:    ${BLUE}branding/svg/${NC}"
echo -e "  📁 PNG exports:    ${BLUE}branding/png/${NC}"
echo -e "  📁 Tauri icons:    ${BLUE}src-tauri/icons/${NC}"
echo -e "  📁 Web favicons:   ${BLUE}public/${NC}"
echo -e "  📁 Social media:   ${BLUE}branding/social/${NC}"
echo ""
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
