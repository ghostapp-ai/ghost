---
title: "Changelog"
description: "Release notes and version history for Ghost Agent OS."
---

## [0.12.0](https://github.com/ghostapp-ai/ghost/compare/v0.11.2...v0.12.0) (2026-02-21)

### 🚀 Features

* **mcp:** add curated MCP tool catalog with one-click install ([8d54189](https://github.com/ghostapp-ai/ghost/commit/8d54189889f40c19510677cd7c8dd26d2a243011))

### 🐛 Bug Fixes

* **agent:** make test_agent_run_emits_events robust for both CI and local ([f955c25](https://github.com/ghostapp-ai/ghost/commit/f955c25a5718c4f4b3c07a8a7d2ab858f2233e65))
* **deps:** update ghost package version to 0.11.2 ([32df794](https://github.com/ghostapp-ai/ghost/commit/32df794dea9e27ca34dd217fc080e7e5552c9c9f))
* **inference:** chunked batch prefill for prompts exceeding BATCH_SIZE ([397743b](https://github.com/ghostapp-ai/ghost/commit/397743b8dbce4bd6fe84a8607621d324b964cf0a))

### ♻️ Code Refactoring

* **mcp:** integrate McpAppStore into Settings MCP tab ([a10c58e](https://github.com/ghostapp-ai/ghost/commit/a10c58ebe78f6c584dc3c3d86216efbb54469c0d))

### 📚 Documentation

* **seo:** position Ghost as complete 2026 agent protocol stack ([6a06697](https://github.com/ghostapp-ai/ghost/commit/6a06697bccca470e43745229f9231c4772e7da9f))
* update core documents for MCP catalog and chunked prefill fix ([195d29f](https://github.com/ghostapp-ai/ghost/commit/195d29f6063fe97ac07a73af7655f3fc5d6bcc60))
* **website:** auto-sync content from source files [skip ci] ([800e850](https://github.com/ghostapp-ai/ghost/commit/800e850df91f3bd9626c5a1b237d7352dd3f9ae6))

## [0.11.2](https://github.com/ghostapp-ai/ghost/compare/v0.11.1...v0.11.2) (2026-02-20)

### 🐛 Bug Fixes

* **agent:** share LlamaBackend singleton between chat and agent ([aea604f](https://github.com/ghostapp-ai/ghost/commit/aea604f1463a49f7ee3ae18e2cd929bac2159bf4))
* **ci:** fix cargo fmt + add lefthook pre-commit hooks ([927a4e3](https://github.com/ghostapp-ai/ghost/commit/927a4e36fd94fe775d2eb551b0634a5742586c6f))
* **docs:** update supported versions in privacy documentation ([97a901a](https://github.com/ghostapp-ai/ghost/commit/97a901a6adf571cce563475dd2f011166b40ff93))

### ⚡ Performance

* **ci:** optimize CI pipeline — reduce build times by ~30% ([9d2c81a](https://github.com/ghostapp-ai/ghost/commit/9d2c81aae47a67ba559d6c0f0e407743445dea70))

### ♻️ Code Refactoring

* **website:** reposition as Agent OS category leader, remove competitor comparisons ([765f358](https://github.com/ghostapp-ai/ghost/commit/765f358c512f4a19794b4c4b183e058cea0d8859))

### 📚 Documentation

* **website:** auto-sync content from source files [skip ci] ([7b33b72](https://github.com/ghostapp-ai/ghost/commit/7b33b7262078b71f99478ff299a34d6b87a79e0e))
* **website:** auto-sync content from source files [skip ci] ([79e09ce](https://github.com/ghostapp-ai/ghost/commit/79e09ceb7246c34f705be5039099438ff106865e))

## [0.11.1](https://github.com/ghostapp-ai/ghost/compare/v0.11.0...v0.11.1) (2026-02-20)

### 🐛 Bug Fixes

* **updater:** regenerate signing keypair with proper password ([442ae26](https://github.com/ghostapp-ai/ghost/commit/442ae2615024f907374d280c2b6965cd78d5952c))

## [0.11.0](https://github.com/ghostapp-ai/ghost/compare/v0.10.1...v0.11.0) (2026-02-20)

### 🚀 Features

* **agent:** add native ReAct agent engine with 104 tests ([0927c1f](https://github.com/ghostapp-ai/ghost/commit/0927c1fe48fa8e723ca0980ca877041ac858c3be))
* **updater:** add frontend auto-update UI with progress tracking ([b4fb0d1](https://github.com/ghostapp-ai/ghost/commit/b4fb0d1bf26b251d04b594214b3e3d3b3e921500))
* **updater:** add tauri-plugin-updater for in-app auto-updates ([220d242](https://github.com/ghostapp-ai/ghost/commit/220d24251b05a96407b5177ff9dab6da9e0a91d5))
* **website:** add Astro Starlight docs site + AI agent automation ([0126732](https://github.com/ghostapp-ai/ghost/commit/012673263aa46b4fefb59ab32ddab148cf8a6b8a))

### 🐛 Bug Fixes

* **agent:** replace loop/match with while-let for clippy compliance ([bdce1e5](https://github.com/ghostapp-ai/ghost/commit/bdce1e5345789ee1bc28bef8cd2eef4ac0fe2b2a))
* **ci:** add id-token: write permission for claude-code-action OIDC ([9f5365d](https://github.com/ghostapp-ai/ghost/commit/9f5365db45a3321912cfc663bb98dc39e8b130d4))
* **ui:** resolve duplicate tray icon and add custom window controls ([d1141a5](https://github.com/ghostapp-ai/ghost/commit/d1141a53451525043a8abae5f2deb575eac5a95b))
* **updater:** update pubkey after key regeneration with --ci flag ([e6e6051](https://github.com/ghostapp-ai/ghost/commit/e6e6051887991fdfb28e066d97768ff480d34e84))

### ♻️ Code Refactoring

* remove pro/ stubs and pro CI step CI action from public repo ([c048d27](https://github.com/ghostapp-ai/ghost/commit/c048d27ffd41e372806104b6fdbe06d6918df8ca))
* replace Claude agents with Copilot-only architecture ([194db99](https://github.com/ghostapp-ai/ghost/commit/194db996e9313d7dbf08253f709db98aab061951))

### 📚 Documentation

* document auto-updater in ROADMAP and CLAUDE decision log ([333d444](https://github.com/ghostapp-ai/ghost/commit/333d444c51d351e40f94d573f1eb647cefb8865d))
* **website:** auto-sync content from source files [skip ci] ([2d00863](https://github.com/ghostapp-ai/ghost/commit/2d008630debc4aec6fa78ac988e3d81d6a5ec33f))

## [0.2.0](https://github.com/ghostapp-ai/ghost/compare/v0.1.1...v0.2.0) (2026-02-20)

### 🚀 Features

* add system tray, filesystem browser, OneDrive detection, and Settings redesign ([717133d](https://github.com/ghostapp-ai/ghost/commit/717133dcb1e967c8f79020e39635c8c6cdc747da))
* **agent:** add native ReAct agent engine with 104 tests ([0927c1f](https://github.com/ghostapp-ai/ghost/commit/0927c1fe48fa8e723ca0980ca877041ac858c3be))
* **chat:** enhance GPU support with runtime auto-detection and update model loading logic ([ce031e8](https://github.com/ghostapp-ai/ghost/commit/ce031e86a9d1b393e5657434050114f498f7f56a))
* **ci:** build iOS unsigned xcarchive without Apple Developer account ([f3f51d8](https://github.com/ghostapp-ai/ghost/commit/f3f51d81fc37896b4e5cd24f4842532cb5ba7e81))
* **ci:** enhance Apple and Android signing process with environment variable checks ([899eb8d](https://github.com/ghostapp-ai/ghost/commit/899eb8d576a9ef80bda07c4ed953298f83e73967))
* **ci:** multiplatform CI/CD — Android, iOS, RPM, DRY pro stub ([5a21cbc](https://github.com/ghostapp-ai/ghost/commit/5a21cbcf0cf2acec3d73da6ce635aca76653ed6f))
* **ci:** replace Release Please with semantic-release for fully automatic releases ([79d76d2](https://github.com/ghostapp-ai/ghost/commit/79d76d2bd83aacf0d30d704f1a45f7738dc72fac))
* **installer:** enhance cross-platform bundler configuration ([4ab5bfa](https://github.com/ghostapp-ai/ghost/commit/4ab5bfa601f63e540a03ef10b178bd6b54bf94c2))
* multiplatform support — Android APK build, conditional compilation, responsive UI ([d00ed4c](https://github.com/ghostapp-ai/ghost/commit/d00ed4cce235a5b463d6f65ba40628c179d6e227))
* **protocols:** implement A2UI v0.9 generative UI renderer ([ab4835a](https://github.com/ghostapp-ai/ghost/commit/ab4835acaeac4e25c9963bdf9ae3e9c0fb8c6b42))
* **protocols:** implement AG-UI event system and streaming chat ([7cc0c6e](https://github.com/ghostapp-ai/ghost/commit/7cc0c6ead2238c42359496abceeb34e0b2df226b))
* **protocols:** implement MCP Client host for external servers ([4e4a0b4](https://github.com/ghostapp-ai/ghost/commit/4e4a0b454083442681ba642db4a5a76185d0f9c5))
* **protocols:** implement MCP Server with 3 tools via rmcp v0.16 ([e99bffb](https://github.com/ghostapp-ai/ghost/commit/e99bffb8925be84b70378df49037145e8ce18831))
* **setup:** add first-launch onboarding wizard with model auto-download ([d16a85d](https://github.com/ghostapp-ai/ghost/commit/d16a85d4ca5c661cb2eca0180a11f8b003b70b21))
* **ui:** add MCP tab to Settings for server management ([7129ec5](https://github.com/ghostapp-ai/ghost/commit/7129ec5aafa47337eb7b1ebdb2730624107e4c5e))
* **updater:** add frontend auto-update UI with progress tracking ([b4fb0d1](https://github.com/ghostapp-ai/ghost/commit/b4fb0d1bf26b251d04b594214b3e3d3b3e921500))
* **updater:** add tauri-plugin-updater for in-app auto-updates ([220d242](https://github.com/ghostapp-ai/ghost/commit/220d24251b05a96407b5177ff9dab6da9e0a91d5))
* **website:** add Astro Starlight docs site + AI agent automation ([0126732](https://github.com/ghostapp-ai/ghost/commit/012673263aa46b4fefb59ab32ddab148cf8a6b8a))

### 🐛 Bug Fixes

* **agent:** replace loop/match with while-let for clippy compliance ([bdce1e5](https://github.com/ghostapp-ai/ghost/commit/bdce1e5345789ee1bc28bef8cd2eef4ac0fe2b2a))
* cargo fmt and bump MSRV to 1.80 for LazyLock support ([76d4b11](https://github.com/ghostapp-ai/ghost/commit/76d4b11263797d95e2c926f7ad40205a29739289))
* **chat:** cap model at 1.5B on CPU, add smart greeting and periodic re-indexing ([f22e937](https://github.com/ghostapp-ai/ghost/commit/f22e937124d6ada75f72d2ea418923445a3cd1a7))
* **ci:** add id-token: write permission for claude-code-action OIDC ([9f5365d](https://github.com/ghostapp-ai/ghost/commit/9f5365db45a3321912cfc663bb98dc39e8b130d4))
* **ci:** add pro stub to build job for cross-platform builds ([38fee37](https://github.com/ghostapp-ai/ghost/commit/38fee37df81df4fea73aa194c67cb2d167dac242))
* **ci:** force dynamic CRT (/MD) for Windows build to resolve LNK2005 ([6ba16b9](https://github.com/ghostapp-ai/ghost/commit/6ba16b9ce3b7ba28d52ec327f8c22c5721b46086))
* **ci:** format stub crate code to pass cargo fmt check ([763bc59](https://github.com/ghostapp-ai/ghost/commit/763bc5971e80cbac992bc4514a80e83903707797))
* **ci:** handle Dependabot PRs and harden repo configuration ([5fa3882](https://github.com/ghostapp-ai/ghost/commit/5fa3882778488d87d4a9291c2df61d6834af3054))
* **ci:** remove dev dep opt-level=2, add mold RUSTFLAGS for Linux ([41da74c](https://github.com/ghostapp-ai/ghost/commit/41da74c1ba86ec814b3b4bac05463fe3bcf1d873))
* **ci:** replace invalid secrets context in step if conditions ([6215840](https://github.com/ghostapp-ai/ghost/commit/6215840c72548196afe738e745924821766a7fae))
* **ci:** resolve all GitHub Actions failures ([a0744eb](https://github.com/ghostapp-ai/ghost/commit/a0744eb2d7fc9703bbd1e38449dc96d1bd2bf7d8))
* **ci:** stub pro crate when private submodule unavailable ([e75583e](https://github.com/ghostapp-ai/ghost/commit/e75583eaaeda29ccad2f20a2fc2088f2cc9c580d))
* **ci:** use ad-hoc macOS signing when no Apple Developer cert configured ([206a2a8](https://github.com/ghostapp-ai/ghost/commit/206a2a8904b2f7d4a347c061e3c802fb0b51265f))
* **ci:** use correct sccache-action version v0.0.9 ([4842405](https://github.com/ghostapp-ai/ghost/commit/4842405e5382cc748c03c9aab7278709279de2dc))
* **tls:** replace aws-lc-rs with ring to fix MSVC Windows build ([82f81ff](https://github.com/ghostapp-ai/ghost/commit/82f81ffe8a4dd4ca15d9b5624ddc8a8baa9c03cd))
* **ui:** resolve duplicate tray icon and add custom window controls ([d1141a5](https://github.com/ghostapp-ai/ghost/commit/d1141a53451525043a8abae5f2deb575eac5a95b))
* **updater:** update pubkey after key regeneration with --ci flag ([e6e6051](https://github.com/ghostapp-ai/ghost/commit/e6e6051887991fdfb28e066d97768ff480d34e84))
* **window:** prevent window from hiding immediately on startup ([dabd993](https://github.com/ghostapp-ai/ghost/commit/dabd99340abe2c7284725b250247b5cf6779a5fb))
* **window:** prevent window from hiding immediately on startup ([894fc66](https://github.com/ghostapp-ai/ghost/commit/894fc66740d2a251d5b73d688d8b07ef9e095b4e))

### ⚡ Performance

* **ci:** optimize build times with cargo profiles, mold linker, and caching ([dc31027](https://github.com/ghostapp-ai/ghost/commit/dc310279f7b83c8168543521b96e513da0475d9f))
* **ci:** parallelize CI jobs and add sccache + nextest ([176e2cf](https://github.com/ghostapp-ai/ghost/commit/176e2cf103e8b2ed7a2e0dd2043c5de23a825d6a))

### ♻️ Code Refactoring

* remove pro/ stubs and pro CI step CI action from public repo ([c048d27](https://github.com/ghostapp-ai/ghost/commit/c048d27ffd41e372806104b6fdbe06d6918df8ca))
* replace Claude agents with Copilot-only architecture ([194db99](https://github.com/ghostapp-ai/ghost/commit/194db996e9313d7dbf08253f709db98aab061951))

### 📚 Documentation

* document auto-updater in ROADMAP and CLAUDE decision log ([333d444](https://github.com/ghostapp-ai/ghost/commit/333d444c51d351e40f94d573f1eb647cefb8865d))
* update all core documents for Agent OS vision with protocol architecture ([96747a1](https://github.com/ghostapp-ai/ghost/commit/96747a1b6e388b4e7209dddf0015fbd265109ca4))
* update CLAUDE.md and ROADMAP.md for open source configuration ([48a1018](https://github.com/ghostapp-ai/ghost/commit/48a1018e967fc271895af814cc96420950fe022a))
* update CLAUDE.md decision log and ROADMAP.md progress ([917809f](https://github.com/ghostapp-ai/ghost/commit/917809fbb9d5e4ee29bd3081a9700c62bb6e3c76))
* update README and CLAUDE for AG-UI completion status ([79cf447](https://github.com/ghostapp-ai/ghost/commit/79cf4472c84149e1519ae29bdf4c4e2551e2df64))
* update README, ROADMAP, and CLAUDE.md with new features ([c2b6c94](https://github.com/ghostapp-ai/ghost/commit/c2b6c94e5f3dd103b91dbad8ae6db6191edc3d0e))
* update ROADMAP and CLAUDE for Phase 1.5 MCP completion ([decf772](https://github.com/ghostapp-ai/ghost/commit/decf7721745e108ecb47527347b7d635e696e570))
* **website:** auto-sync content from source files [skip ci] ([2d00863](https://github.com/ghostapp-ai/ghost/commit/2d008630debc4aec6fa78ac988e3d81d6a5ec33f))

## [0.10.1](https://github.com/ghostapp-ai/ghost/compare/v0.10.0...v0.10.1) (2026-02-20)

### 🐛 Bug Fixes

* **tls:** replace aws-lc-rs with ring to fix MSVC Windows build ([b157fe3](https://github.com/ghostapp-ai/ghost/commit/b157fe3d08671871cfd16cf2000857fe879c59b0))

## [0.10.0](https://github.com/ghostapp-ai/ghost/compare/v0.9.2...v0.10.0) (2026-02-20)

### 🚀 Features

* **protocols:** implement A2UI v0.9 generative UI renderer ([3a10674](https://github.com/ghostapp-ai/ghost/commit/3a10674c79ceaa9ffb2e69dbdd3e1a396a6e92e5))

## [unreleased]

### 🚀 Features

* **protocols:** implement A2UI v0.9 generative UI — Rust types + React renderer + AG-UI integration
  - Rust `protocols/a2ui.rs`: full A2UI v0.9 types, component builders, AG-UI bridge, 8 tests
  - React `A2UIRenderer.tsx`: 17+ component types (Text, Button, TextField, Card, Row, Column, etc.)
  - Data binding via JSON Pointers (RFC 6901) with two-way input support
  - Adjacency list → tree resolution with automatic root detection
  - A2UI surfaces transported via AG-UI CUSTOM events over Tauri IPC
  - `useAgui` hook processes createSurface/updateComponents/updateDataModel/deleteSurface
  - +9.8 KB JS bundle cost (245.8 KB total)

## [0.9.2](https://github.com/ghostapp-ai/ghost/compare/v0.9.1...v0.9.2) (2026-02-20)

### 🐛 Bug Fixes

* **ci:** resolve all GitHub Actions failures ([42ee433](https://github.com/ghostapp-ai/ghost/commit/42ee43301816937ee415f20a925b17846c88e463))

## [0.9.1](https://github.com/ghostapp-ai/ghost/compare/v0.9.0...v0.9.1) (2026-02-20)

### 🐛 Bug Fixes

* **ci:** handle Dependabot PRs and harden repo configuration ([c30b301](https://github.com/ghostapp-ai/ghost/commit/c30b301caab7417cce721f7422aad64723b1619f))

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
