---
title: Multiplatform
description: How Ghost runs on Windows, macOS, Linux, Android, and iOS from one codebase.
---

Ghost targets 5 platforms from a single codebase using Tauri v2's conditional compilation.

## Platform Matrix

| Platform | Status | Build Target | Package |
|----------|--------|-------------|---------|
| Windows x64 | ✅ | `x86_64-pc-windows-msvc` | NSIS installer |
| macOS ARM64 | ✅ | `aarch64-apple-darwin` | DMG |
| macOS Intel | ✅ | `x86_64-apple-darwin` | DMG |
| Linux x64 | ✅ | `x86_64-unknown-linux-gnu` | DEB, RPM, AppImage |
| Android ARM64 | ✅ | `aarch64-linux-android` | APK |
| iOS ARM64 | 🔧 | `aarch64-apple-ios` | xcarchive |

## Conditional Compilation

Ghost uses Tauri's `#[cfg(desktop)]` and `#[cfg(mobile)]` macros:

```rust
// Desktop-only: file watcher, system tray, MCP stdio
#[cfg(desktop)]
pub fn start_file_watcher() -> Result<()> { ... }

// Mobile stub: return Ok or clear error
#[cfg(mobile)]
pub fn start_file_watcher() -> Result<()> {
    Ok(()) // Not supported on mobile
}
```

### Desktop-Only Features

- File system watcher (`notify` crate)
- System tray icon + menu
- Global keyboard shortcuts
- MCP stdio transport
- Native chat engine (llama-cpp-2)

### Mobile Adaptations

- 44px+ touch targets (`@media (pointer: coarse)`)
- Safe area padding for notch/home indicator
- `h-dvh` instead of `h-screen` for dynamic viewport
- Full-screen settings (no floating modals)
- Platform detection via `usePlatform()` hook

## TLS Strategy

Ghost uses **100% rustls** — no OpenSSL:

```toml
# Cargo.toml — all HTTP crates use rustls
reqwest = { version = "0.12", features = ["rustls-tls"] }
hf-hub = { version = "0.4", features = ["rustls-tls"] }
```

This eliminates Android NDK cross-compilation failures with OpenSSL.

## Capabilities Split

Tauri permissions are split per platform:

- `default.json` — all platforms (core IPC)
- `desktop.json` — file system, tray, shortcuts
- `mobile.json` — haptics, safe area APIs

## CI/CD

All platforms build automatically via GitHub Actions on every release:

- Windows, macOS (ARM+Intel), Linux → Desktop builds
- Android → APK with optional signing
- iOS → Unsigned xcarchive (sideloadable via AltStore)
