# Ghost — Privacy-First AI Desktop Assistant

> v0.14.2 · Tauri v2 + Rust + React/TS/Vite

## What Is Ghost

Ghost is a local-first AI desktop assistant that keeps your data on your machine. It combines full-text search (FTS5, <5ms), semantic vector search (Candle embeddings, all-MiniLM-L6-v2 384D), and a ReAct agent engine with grammar-constrained tool calling — all running without cloud dependencies.

**Core stack:**
- **Backend:** 20,688 lines of Rust — Tauri v2, rusqlite, Candle ML
- **Frontend:** 25 files — React + TypeScript + Vite
- **Local AI:** Qwen2.5-Instruct GGUF (0.5B–7B parameter models)
- **Search:** Hybrid FTS5 + semantic vector, sub-5ms text queries
- **Agent:** ReAct engine with grammar-constrained tool calling
- **Protocols (2026 stack):** MCP Server+Client, AG-UI, A2UI, A2A, Skills
- **MCP Catalog:** 30+ curated servers + 6,000+ from registry
- **File Processing:** Watcher + text extraction for PDF, DOCX, 50+ formats
- **Distribution:** 5+ releases, CI/CD GitHub Actions (Windows/macOS/Linux)

## Market

| Metric | Value |
|---|---|
| AI Desktop Assistant TAM (2025) | $3.35B |
| Projected TAM (2030) | $24.53B |
| CAGR | ~39% |

**Competitor landscape:**

| Competitor | Status | Ghost Advantage |
|---|---|---|
| Rewind AI | Dead (pivoted to Limitless, lost desktop focus) | Ghost is actively maintained, desktop-first |
| Windows Recall | Privacy backlash, enterprise distrust | Ghost is 100% local, zero telemetry |
| Raycast | Mac-only, no local AI | Ghost is cross-platform, runs local models |
| Notion AI / Copilot | Cloud-dependent, data leaves your machine | Ghost keeps everything local |

**Why Ghost wins:** The only cross-platform, privacy-first desktop assistant with local LLM inference, hybrid search, and the full 2026 agent protocol stack (MCP, A2A, AG-UI). No data leaves the machine.

## Pricing

| Tier | Price | Features |
|---|---|---|
| **Free** | $0 | Local AI, FTS5 search, file watcher, basic MCP tools |
| **Pro** | $8/mo | Semantic search, full MCP catalog, advanced agent skills, priority updates |
| **Teams** | $15/user/mo | Shared knowledge bases, team MCP servers, admin controls, SSO |

Monetization via `ghost-pro` crate — feature gating at the Rust level.

## Current Status

- **Version:** 0.14.2
- **Maturity:** Alpha — functional for personal use, not yet monetized
- **Build:** `cargo build --release`
- **Test:** `cargo test`
- **CI/CD:** GitHub Actions multiplatform (Windows, macOS, Linux)
- **Releases:** 5+ published

## Blockers to Revenue

1. **Stripe integration** — No payment processing. Need Stripe Checkout + Customer Portal for Pro/Teams.
2. **License key validation** — No mechanism to gate Pro features. Need license key generation + validation in `ghost-pro` crate.
3. **App store distribution** — Not yet on any store. Need code signing, notarization (macOS), and Microsoft Store submission.
4. **Landing page** — No marketing site. Need ghost.quirozai.com or similar.

## Key Metrics to Track

- Downloads per release (GitHub Releases + app stores)
- Free → Pro conversion rate (target: 5–8%)
- MRR from Pro + Teams subscriptions
- DAU / retention (local telemetry opt-in only)
