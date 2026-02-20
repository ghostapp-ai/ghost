# Copilot Instructions — Ghost Project

## Project Overview
Ghost is a private, local-first **Agent OS** for desktop and mobile. It uses Tauri v2 (Rust backend) + React/TypeScript (frontend) + SQLite/sqlite-vec + Candle (native AI).

## Key Conventions
- **Privacy first**: NEVER add telemetry, analytics, or external network calls
- **Rust backend**: `src-tauri/` with `thiserror`, `anyhow`, `tokio`, `tracing`
- **Frontend**: React 18 + TypeScript strict + Tailwind CSS v4 in `src/`
- **Website**: Astro Starlight in `website/` — deployed to GitHub Pages
- **Commits**: conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`)
- **Never use `--all-features`** in Rust — `metal`/`accelerate` are macOS-only

## Website Maintenance
The documentation website (`website/`) should stay in sync with source files:
- `README.md` → `website/src/content/docs/guides/introduction.md`
- `ROADMAP.md` → `website/src/content/docs/reference/roadmap.md`
- `CHANGELOG.md` → `website/src/content/docs/reference/changelog.md`
- `CONTRIBUTING.md` → `website/src/content/docs/reference/contributing.md`

Use the `website-maintainer` custom agent for website tasks.

## File Structure
```
src-tauri/src/     # Rust backend (indexer, db, embeddings, chat, agent, protocols)
src/               # React frontend (components, hooks, lib)
website/           # Astro Starlight documentation site
pro/               # Proprietary Pro features (submodule)
scripts/           # Build and maintenance scripts
.github/agents/    # Custom Copilot agents
.claude/agents/    # Claude Code agent definitions
```

## For Detailed Architecture
Read `CLAUDE.md` at the project root for full architecture, conventions, and decision log.
