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
