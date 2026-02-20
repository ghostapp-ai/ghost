# Contributing to Ghost

Thank you for your interest in contributing to Ghost! This guide will help you get started.

## Code of Conduct

Be respectful, constructive, and professional. We're building privacy-first software — treat contributors with the same care we treat user data.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) >= 1.0
- Platform-specific Tauri v2 dependencies: [see guide](https://v2.tauri.app/start/prerequisites/)
- [Ollama](https://ollama.com/) (optional — Ghost uses native AI by default)

### Setup

```bash
# Clone the repo
git clone https://github.com/ghostapp-ai/ghost.git
cd ghost

# Install frontend dependencies
bun install

# Run in development mode
bun run tauri dev

# Run Rust tests
cd src-tauri && cargo test

# Run Rust linter
cd src-tauri && cargo clippy -- -D warnings

# Build frontend
bun run build
```

## Development Workflow

### 1. Branch Naming

```
feature/search-bar
feature/file-watcher
fix/fts5-unicode-tokenizer
refactor/db-connection-pool
docs/update-roadmap
```

### 2. Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

**Types**: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`

**Examples**:
```
feat(search): implement hybrid FTS5 + vector search with RRF ranking
fix(indexer): handle PDF files with encrypted content gracefully
docs(readme): update architecture diagram
```

### 3. Before Submitting a PR

- [ ] All Rust tests pass: `cd src-tauri && cargo test`
- [ ] No clippy warnings: `cd src-tauri && cargo clippy -- -D warnings`
- [ ] Frontend compiles: `bun run build`
- [ ] No `unwrap()` in production code (only in tests)
- [ ] Public functions have doc comments
- [ ] Updated `ROADMAP.md` if completing a milestone

### 4. PR Guidelines

- Fill out the PR template completely.
- One logical change per PR — keep them focused.
- Link to relevant issues.
- Include screenshots for UI changes.
- Keep the privacy rules: **NEVER** add telemetry, analytics, or external network calls.

## Architecture Overview

Read [CLAUDE.md](CLAUDE.md) for the complete architecture guide, conventions, and module structure.

**Key rules**:
- All heavy logic lives in Rust — frontend is a thin UI layer.
- Database access through connection pool — no connections held across await points.
- Use `thiserror` for library errors, `anyhow` for application errors.
- Use `tracing` for logging — never `println!`.
- TypeScript strict mode — no `any` types.

## Areas for Contribution

### Good First Issues
Look for issues labeled [`good first issue`](https://github.com/ghostapp-ai/ghost/labels/good%20first%20issue).

### Current Priorities
Check [ROADMAP.md](ROADMAP.md) for the current phase and what needs to be done.

### Key Areas
- **Performance**: Cold start optimization, search latency, memory usage
- **Text Extraction**: Better PDF/DOCX parsing, new format support
- **Search Quality**: Ranking improvements, snippet highlighting
- **Cross-Platform**: macOS/Linux-specific fixes
- **Documentation**: Code comments, guides, tutorials

## Privacy Rules (Non-Negotiable)

These rules CANNOT be violated in any contribution:

1. **NEVER** add telemetry, analytics, or crash reporting.
2. **NEVER** make external network calls (except localhost Ollama + one-time HuggingFace model download).
3. **NEVER** include tracking pixels or third-party SDKs that phone home.
4. All data processing MUST happen locally.
5. If a feature requires cloud access, it MUST be opt-in and clearly documented.

PRs violating these rules will be rejected immediately.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
