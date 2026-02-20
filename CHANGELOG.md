## [0.9.0](https://github.com/ghostapp-ai/ghost/compare/v0.8.0...v0.9.0) (2026-02-20)

### 🚀 Features

* **ci:** enhance Apple and Android signing process with environment variable checks ([0931516](https://github.com/ghostapp-ai/ghost/commit/0931516aec344e5a55856938c27312eea6a8b289))

## [0.2.0](https://github.com/ghostapp-ai/ghost/compare/v0.1.1...v0.2.0) (2026-02-20)

### 🚀 Features

* add system tray, filesystem browser, OneDrive detection, and Settings redesign ([f175be6](https://github.com/ghostapp-ai/ghost/commit/f175be6e504ada652f428b361d90573de940fdc2))
* **chat:** enhance GPU support with runtime auto-detection and update model loading logic ([3809cc5](https://github.com/ghostapp-ai/ghost/commit/3809cc506ab1295aafb5a6021c7729d833708be1))
* **ci:** build iOS unsigned xcarchive without Apple Developer account ([4f44b80](https://github.com/ghostapp-ai/ghost/commit/4f44b80f263398063dae4beabdefdfa5642131da))
* **ci:** enhance Apple and Android signing process with environment variable checks ([0931516](https://github.com/ghostapp-ai/ghost/commit/0931516aec344e5a55856938c27312eea6a8b289))
* **ci:** multiplatform CI/CD — Android, iOS, RPM, DRY pro stub ([8471e5b](https://github.com/ghostapp-ai/ghost/commit/8471e5bcf80e20ecc99c2b8ba8373a08631a524b))
* **ci:** replace Release Please with semantic-release for fully automatic releases ([79d76d2](https://github.com/ghostapp-ai/ghost/commit/79d76d2bd83aacf0d30d704f1a45f7738dc72fac))
* **installer:** enhance cross-platform bundler configuration ([f93a821](https://github.com/ghostapp-ai/ghost/commit/f93a821b9cf1e35d808cd736c8c6de8bf9954f92))
* multiplatform support — Android APK build, conditional compilation, responsive UI ([ee72542](https://github.com/ghostapp-ai/ghost/commit/ee72542f61d8c6b262fdee90ce83d4701e3c69c7))
* **protocols:** implement AG-UI event system and streaming chat ([2fefd74](https://github.com/ghostapp-ai/ghost/commit/2fefd7409cf682965082878b365f390a294d23f3))
* **protocols:** implement MCP Client host for external servers ([c054c2e](https://github.com/ghostapp-ai/ghost/commit/c054c2e979375549328b599a7e7dadcd6391157a))
* **protocols:** implement MCP Server with 3 tools via rmcp v0.16 ([31b3e4a](https://github.com/ghostapp-ai/ghost/commit/31b3e4acc4fdfd5a3ad4bbbbc9bfbb066e798538))
* **setup:** add first-launch onboarding wizard with model auto-download ([ac687f4](https://github.com/ghostapp-ai/ghost/commit/ac687f41b8fd1081891197067b022433904d350d))
* **ui:** add MCP tab to Settings for server management ([aba8fc7](https://github.com/ghostapp-ai/ghost/commit/aba8fc78a693c2761f7d0cb90c290cdf4d247bd9))

### 🐛 Bug Fixes

* cargo fmt and bump MSRV to 1.80 for LazyLock support ([976eaeb](https://github.com/ghostapp-ai/ghost/commit/976eaeb1b48c36eb34a7552f97ada6a887984b74))
* **chat:** cap model at 1.5B on CPU, add smart greeting and periodic re-indexing ([5aeb901](https://github.com/ghostapp-ai/ghost/commit/5aeb901bcbd557c93f14e12a235604cf334feb90))
* **ci:** add pro stub to build job for cross-platform builds ([1cf1f50](https://github.com/ghostapp-ai/ghost/commit/1cf1f5099737bd6f3ffc152e088c80a9750bd41d))
* **ci:** force dynamic CRT (/MD) for Windows build to resolve LNK2005 ([844b07d](https://github.com/ghostapp-ai/ghost/commit/844b07d5a0cb57781d2ff0772d5b809f5e787f79))
* **ci:** format stub crate code to pass cargo fmt check ([df84876](https://github.com/ghostapp-ai/ghost/commit/df84876d5f4b81a26aa482f21b86642d209d168d))
* **ci:** remove dev dep opt-level=2, add mold RUSTFLAGS for Linux ([4b87a6a](https://github.com/ghostapp-ai/ghost/commit/4b87a6a207889335bbadaa73549f4334b4d7e6e9))
* **ci:** replace invalid secrets context in step if conditions ([d181b96](https://github.com/ghostapp-ai/ghost/commit/d181b965d0a83cf2cdebbcdf242bdd507811736c))
* **ci:** stub pro crate when private submodule unavailable ([948158a](https://github.com/ghostapp-ai/ghost/commit/948158ae0433abbf3b7434ed14a4f15a32f17a23))
* **ci:** use ad-hoc macOS signing when no Apple Developer cert configured ([eb84cb0](https://github.com/ghostapp-ai/ghost/commit/eb84cb0e764566f440c46a9a04c992d0d32ecd8c))
* **ci:** use correct sccache-action version v0.0.9 ([2222575](https://github.com/ghostapp-ai/ghost/commit/222257594b3327365e896aeae17b817e8f93c953))
* **window:** prevent window from hiding immediately on startup ([96b31a8](https://github.com/ghostapp-ai/ghost/commit/96b31a88403c8af07243a82e432e8255ba4419ba))
* **window:** prevent window from hiding immediately on startup ([0184263](https://github.com/ghostapp-ai/ghost/commit/0184263c91eab14dbad9b67ff452c233c47bcc95))

### ⚡ Performance

* **ci:** optimize build times with cargo profiles, mold linker, and caching ([f3d49a0](https://github.com/ghostapp-ai/ghost/commit/f3d49a0b804ba97d813370ee0342915a62d9b11e))
* **ci:** parallelize CI jobs and add sccache + nextest ([6795a6b](https://github.com/ghostapp-ai/ghost/commit/6795a6b37b0b52ee9d208ec5cdad377b43bd2492))

### 📚 Documentation

* update all core documents for Agent OS vision with protocol architecture ([4eb3420](https://github.com/ghostapp-ai/ghost/commit/4eb34208e43ab190edaf041b2614b5c216c1a267))
* update CLAUDE.md and ROADMAP.md for open source configuration ([b7985af](https://github.com/ghostapp-ai/ghost/commit/b7985af45976313f51e0ce65f5f45ee1aa66f477))
* update CLAUDE.md decision log and ROADMAP.md progress ([4af23fc](https://github.com/ghostapp-ai/ghost/commit/4af23fcd1414f1d1de69e7a1179ea21628dfd0b6))
* update README and CLAUDE for AG-UI completion status ([80c21e8](https://github.com/ghostapp-ai/ghost/commit/80c21e844dbe0aff8b4bc738bef7d5abe2bd900d))
* update README, ROADMAP, and CLAUDE.md with new features ([2b7b3cd](https://github.com/ghostapp-ai/ghost/commit/2b7b3cd1dd65090f7334fe85a70ae067c85326ad))
* update ROADMAP and CLAUDE for Phase 1.5 MCP completion ([1d6228b](https://github.com/ghostapp-ai/ghost/commit/1d6228bfb18cd5ad5e4a973164ab008502b23914))

## [0.8.0](https://github.com/ghostapp-ai/ghost/compare/v0.7.2...v0.8.0) (2026-02-20)

### 🚀 Features

* **ci:** build iOS unsigned xcarchive without Apple Developer account ([c618cbe](https://github.com/ghostapp-ai/ghost/commit/c618cbe43f7671c66c9638bc1c968b5a271bfab8))

### 🐛 Bug Fixes

* **ci:** replace invalid secrets context in step if conditions ([09c7dcf](https://github.com/ghostapp-ai/ghost/commit/09c7dcfbb0e399f51327d7e78c3b8903ef5e70af))

## [0.7.2](https://github.com/ghostapp-ai/ghost/compare/v0.7.1...v0.7.2) (2026-02-20)

### 🐛 Bug Fixes

* **ci:** force dynamic CRT (/MD) for Windows build to resolve LNK2005 ([6345b7f](https://github.com/ghostapp-ai/ghost/commit/6345b7f22a1414f724133033f21ca2ad9bfdba2a))

## [0.7.1](https://github.com/ghostapp-ai/ghost/compare/v0.7.0...v0.7.1) (2026-02-20)

### 🐛 Bug Fixes

* **ci:** use ad-hoc macOS signing when no Apple Developer cert configured ([e3b9af0](https://github.com/ghostapp-ai/ghost/commit/e3b9af05bdcc4c8929cb797374b908275f65a639))

## [0.7.0](https://github.com/ghostapp-ai/ghost/compare/v0.6.0...v0.7.0) (2026-02-20)

### 🚀 Features

* **ci:** multiplatform CI/CD — Android, iOS, RPM, DRY pro stub ([a5d69e7](https://github.com/ghostapp-ai/ghost/commit/a5d69e776ba9d3b8aa32366a6bb7071e49f98b10))
* multiplatform support — Android APK build, conditional compilation, responsive UI ([6894842](https://github.com/ghostapp-ai/ghost/commit/689484299149196bcdb44baa34bec6b1210ecb20))

### 📚 Documentation

* update README and CLAUDE for AG-UI completion status ([3942a47](https://github.com/ghostapp-ai/ghost/commit/3942a47ed16e730aef4dc41a9cdb75791d160346))

## [0.6.0](https://github.com/ghostapp-ai/ghost/compare/v0.5.0...v0.6.0) (2026-02-19)

### 🚀 Features

* **protocols:** implement AG-UI event system and streaming chat ([2532558](https://github.com/ghostapp-ai/ghost/commit/2532558e2e20723e7418f3014791882773d13707))

### 🐛 Bug Fixes

* **ci:** use correct sccache-action version v0.0.9 ([0ee1d48](https://github.com/ghostapp-ai/ghost/commit/0ee1d48111c75d95069625d31e20e5bc6d82a1ce))

### ⚡ Performance

* **ci:** parallelize CI jobs and add sccache + nextest ([cf824ae](https://github.com/ghostapp-ai/ghost/commit/cf824aeed37e699a1d6fdedcb4e2ce4fdaf3d9bd))

### 📚 Documentation

* update CLAUDE.md decision log and ROADMAP.md progress ([b14abc1](https://github.com/ghostapp-ai/ghost/commit/b14abc1d971c65164765c5cdc5ebacf0e6a80059))

## [0.5.0](https://github.com/ghostapp-ai/ghost/compare/v0.4.0...v0.5.0) (2026-02-19)

### 🚀 Features

* **protocols:** implement MCP Client host for external servers ([7a6f75e](https://github.com/ghostapp-ai/ghost/commit/7a6f75e75b710f2e0676d51de79d988cad32da79))
* **protocols:** implement MCP Server with 3 tools via rmcp v0.16 ([dbc9334](https://github.com/ghostapp-ai/ghost/commit/dbc93347c07ef549b8488bd3f7ec26a85834ba92))
* **ui:** add MCP tab to Settings for server management ([af469cf](https://github.com/ghostapp-ai/ghost/commit/af469cf3018f846a749f5f6484c0550dec1d8d44))

### 📚 Documentation

* update ROADMAP and CLAUDE for Phase 1.5 MCP completion ([b8d0ed5](https://github.com/ghostapp-ai/ghost/commit/b8d0ed596aa24dac23f85554cc357d27c83aa342))

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
