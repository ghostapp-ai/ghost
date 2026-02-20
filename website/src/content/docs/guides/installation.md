---
title: "Download & Install"
description: "Download Ghost for Windows, macOS, Linux, or Android. No external dependencies, no GPU, no cloud account required."
---

## Download Ghost

Download the latest release from [**GitHub Releases**](https://github.com/ghostapp-ai/ghost/releases/latest). No Ollama, no GPU, no internet required after first launch.

### Desktop

| Platform | Download | Details |
| --- | --- | --- |
| **Windows** x64 | [`ghost_x.x.x_x64-setup.exe`](https://github.com/ghostapp-ai/ghost/releases/latest) | NSIS installer · No admin required · WebView2 auto-bootstrap |
| **macOS** Apple Silicon | [`ghost_x.x.x_aarch64.dmg`](https://github.com/ghostapp-ai/ghost/releases/latest) | M1/M2/M3/M4 · DMG with drag-to-install |
| **macOS** Intel | [`ghost_x.x.x_x64.dmg`](https://github.com/ghostapp-ai/ghost/releases/latest) | Intel-based Macs · macOS 10.15+ |
| **Linux** DEB | [`ghost_x.x.x_amd64.deb`](https://github.com/ghostapp-ai/ghost/releases/latest) | Debian · Ubuntu · Pop!_OS · Mint |
| **Linux** RPM | [`ghost_x.x.x_x86_64.rpm`](https://github.com/ghostapp-ai/ghost/releases/latest) | Fedora · RHEL · CentOS · openSUSE |
| **Linux** Universal | [`ghost_x.x.x_amd64.AppImage`](https://github.com/ghostapp-ai/ghost/releases/latest) | Any distro · No install needed |

### Mobile

| Platform | Download | Details |
| --- | --- | --- |
| **Android** ARM64 | [`Ghost_x.x.x_android-aarch64.apk`](https://github.com/ghostapp-ai/ghost/releases/latest) | Min SDK 24 · Tauri v2 WebView |
| **iOS** | *Coming soon* | xcarchive ready · Requires macOS build |

:::note[Zero Dependencies]
Ghost ships with native AI inference via Candle. No Ollama, no GPU, no internet needed after installation. The first launch downloads a ~23MB embedding model once.
:::

## Linux Quick Install

```bash
# Debian/Ubuntu
wget https://github.com/ghostapp-ai/ghost/releases/latest/download/ghost_0.11.0_amd64.deb
sudo dpkg -i ghost_0.11.0_amd64.deb

# Fedora/RHEL
wget https://github.com/ghostapp-ai/ghost/releases/latest/download/ghost_0.11.0_x86_64.rpm
sudo rpm -i ghost_0.11.0_x86_64.rpm

# AppImage (any distro)
wget https://github.com/ghostapp-ai/ghost/releases/latest/download/ghost_0.11.0_amd64.AppImage
chmod +x ghost_0.11.0_amd64.AppImage
./ghost_0.11.0_amd64.AppImage
```

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) >= 1.0 (or Node.js >= 18)
- Platform-specific Tauri v2 dependencies ([see guide](https://v2.tauri.app/start/prerequisites/))
- [Ollama](https://ollama.com/) (optional — Ghost uses native AI by default)

### Steps

```bash
# Clone the repo
git clone https://github.com/ghostapp-ai/ghost.git
cd ghost

# Install frontend dependencies
bun install

# Run in development mode
# (native AI model downloads on first run, ~23MB)
bun run tauri dev
```

### Build for Production

```bash
# Desktop (Windows, macOS, or Linux — auto-detected)
bun run tauri build

# Android (requires Android SDK + NDK 27+)
bun run tauri android build --target aarch64
```

The desktop installer will be generated in `src-tauri/target/release/bundle/`.

### Optional: Ollama for Higher-Quality Models

```bash
# Higher-quality 768D embeddings (vs 384D native)
ollama pull nomic-embed-text

# Agent reasoning model (for tool calling)
ollama pull qwen3:8b
```

## System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **RAM** | 2 GB | 8 GB+ (for larger chat models) |
| **Storage** | 100 MB | 500 MB+ (with AI models) |
| **OS** | Windows 10, macOS 10.15, Ubuntu 20.04 | Latest versions |
| **CPU** | Any x86_64 or ARM64 | AVX2/NEON SIMD support |
| **GPU** | Not required | CUDA/Metal for faster inference |

## First Launch

On first launch, Ghost will:
1. Show an onboarding wizard with hardware detection
2. Download the native AI model (~23MB) — requires internet once
3. Auto-discover your Documents, Desktop, Downloads, and Pictures folders
4. Start indexing files in the background

Subsequent launches are instant (<500ms) with the cached model.
