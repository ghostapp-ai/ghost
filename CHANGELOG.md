## [0.4.0](https://github.com/ghostapp-ai/ghost/compare/v0.3.0...v0.4.0) (2026-02-19)

### 🚀 Features

* **chat:** enhance GPU support with runtime auto-detection and update model loading logic ([4e52d36](https://github.com/ghostapp-ai/ghost/commit/4e52d3631b4d6b08d8d7f757e008df863a096517))

### 🐛 Bug Fixes

* **chat:** cap model at 1.5B on CPU, add smart greeting and periodic re-indexing ([1232a3c](https://github.com/ghostapp-ai/ghost/commit/1232a3c23a449f214e1c7f01f04cdda8fdd0acb8))

## [0.3.0](https://github.com/ghostapp-ai/ghost/compare/v0.2.4...v0.3.0) (2026-02-19)

### 🚀 Features

* add system tray, filesystem browser, OneDrive detection, and Settings redesign ([7f0e3c1](https://github.com/ghostapp-ai/ghost/commit/7f0e3c158d741bebf227bd6bc988d2f75b24a174))
* **installer:** enhance cross-platform bundler configuration ([01d0b74](https://github.com/ghostapp-ai/ghost/commit/01d0b74523652b8bc50479587e0f5e57a879fb19))
* **setup:** add first-launch onboarding wizard with model auto-download ([2a5a97e](https://github.com/ghostapp-ai/ghost/commit/2a5a97e65c187640eaff77cd3d01bda19d3cd5aa))

### 🐛 Bug Fixes

* **window:** prevent window from hiding immediately on startup ([5e12e33](https://github.com/ghostapp-ai/ghost/commit/5e12e33472998c7423acb038776cdc0164ab6f63))
* **window:** prevent window from hiding immediately on startup ([e136a13](https://github.com/ghostapp-ai/ghost/commit/e136a136b0f00a2917a47b777b69340897458a63))

### 📚 Documentation

* update all core documents for Agent OS vision with protocol architecture ([7168e3a](https://github.com/ghostapp-ai/ghost/commit/7168e3abac04cba837512c45b23440e0cae98a14))
* update README, ROADMAP, and CLAUDE.md with new features ([544e60c](https://github.com/ghostapp-ai/ghost/commit/544e60c30677fcf556e8c33f31c8362d1f88f737))

## [0.2.4](https://github.com/ghostapp-ai/ghost/compare/v0.2.3...v0.2.4) (2026-02-19)

### 🐛 Bug Fixes

* **ci:** remove dev dep opt-level=2, add mold RUSTFLAGS for Linux ([f2f4473](https://github.com/ghostapp-ai/ghost/commit/f2f4473db169f57a9c9bf1adfaa0c680d9ea1d57))

## [0.2.3](https://github.com/ghostapp-ai/ghost/compare/v0.2.2...v0.2.3) (2026-02-19)

### ⚡ Performance

* **ci:** optimize build times with cargo profiles, mold linker, and caching ([17836e5](https://github.com/ghostapp-ai/ghost/commit/17836e5116a6655d364d55598e82d307843a81ee))

## [0.2.2](https://github.com/ghostapp-ai/ghost/compare/v0.2.1...v0.2.2) (2026-02-19)

### 🐛 Bug Fixes

* **ci:** add pro stub to build job for cross-platform builds ([014be57](https://github.com/ghostapp-ai/ghost/commit/014be57ebc642ffff3f187db3f87e3fb3a0c017f))

## [0.2.1](https://github.com/ghostapp-ai/ghost/compare/v0.2.0...v0.2.1) (2026-02-19)

### 🐛 Bug Fixes

* cargo fmt and bump MSRV to 1.80 for LazyLock support ([0425829](https://github.com/ghostapp-ai/ghost/commit/0425829292f90dc031d4298aed6153fb31ffa855))
* **ci:** format stub crate code to pass cargo fmt check ([f5018a6](https://github.com/ghostapp-ai/ghost/commit/f5018a69f5dbb684ee68fcaa6e059e6300170197))
* **ci:** stub pro crate when private submodule unavailable ([c8f9a5c](https://github.com/ghostapp-ai/ghost/commit/c8f9a5c5cc17233358a0471b1ee4423b89de2e0e))

### 📚 Documentation

* update CLAUDE.md and ROADMAP.md for open source configuration ([64cbeab](https://github.com/ghostapp-ai/ghost/commit/64cbeab67a4f7e62f3efa1a6ef020e6cd8349b90))

# Changelog

## [0.2.0](https://github.com/ghostapp-ai/ghost/compare/v0.1.1...v0.2.0) (2026-02-19)

### 🚀 Features

* **ci:** replace Release Please with semantic-release for fully automatic releases

## [0.1.1](https://github.com/ghostapp-ai/ghost/compare/v0.1.0...v0.1.1) (2026-02-19)

### 🚀 Features

* **branding:** add custom Ghost visual identity and app icons
* **chat:** download progress bar with filesystem monitoring
* **ci:** replace manual tag releases with Release Please auto-release pipeline
* **ui:** unified Omnibox — single intelligent search+chat tab
* zero-config auto-indexing + reliable window dragging + 50+ file types
* native chat engine with hardware-aware model auto-selection

### 🐛 Bug Fixes

* **ci:** remove --all-features from test step — objc2 is macOS-only
* **ci:** remove invalid reviewers property from dependabot.yml
* **runtime:** tokio panic, sqlite-vec loading, model 404, hardware polling

## [0.1.0](https://github.com/ghostapp-ai/ghost/releases/tag/v0.1.0) (2026-02-19)

### 🚀 Features

* complete Phase 1 — Spotlight-like search bar with full UX
* Implement native AI embedding engine using Candle
* first version
