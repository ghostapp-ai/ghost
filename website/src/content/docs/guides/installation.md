---
title: Installation
description: Download and install Ghost on Windows, macOS, Linux, or Android.
---

## Download Pre-Built Binaries

Download the latest release from [**GitHub Releases**](https://github.com/ghostapp-ai/ghost/releases/latest):

| Platform | File | Notes |
|----------|------|-------|
| **Windows** (64-bit) | `ghost_x.x.x_x64-setup.exe` | NSIS installer, no admin required |
| **macOS** (Apple Silicon) | `ghost_x.x.x_aarch64.dmg` | M1/M2/M3/M4 Macs |
| **macOS** (Intel) | `ghost_x.x.x_x64.dmg` | Intel-based Macs |
| **Linux** (Debian/Ubuntu) | `ghost_x.x.x_amd64.deb` | DEB package |
| **Linux** (Fedora/RHEL) | `ghost_x.x.x_x86_64.rpm` | RPM package |
| **Linux** (Universal) | `ghost_x.x.x_amd64.AppImage` | No install needed |
| **Android** (ARM64) | `Ghost_x.x.x_android-aarch64.apk` | Min SDK 24 |

:::note
No external dependencies required. Ghost ships with native AI inference — no Ollama, no GPU, no internet needed after installation.
:::

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
