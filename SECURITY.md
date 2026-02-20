# Security Policy

## Ghost's Privacy Commitment

Ghost is a **100% local-first** application. Your data never leaves your machine.

- **Zero telemetry**: No usage data, analytics, or crash reports are collected.
- **Zero cloud**: All AI inference runs locally (native Candle engine or optional Ollama).
- **Zero network**: The only network calls are to `localhost` (Ollama) and a one-time model download from HuggingFace Hub on first launch.
- **Single-file database**: Your entire vault is one `.db` file you control.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in Ghost, please report it responsibly:

1. **DO NOT** create a public GitHub issue for security vulnerabilities.
2. Email: **security@ghost-app.dev** (or create a private advisory on GitHub).
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Impact assessment
   - Suggested fix (if any)

We will acknowledge your report within **48 hours** and provide a fix timeline within **7 days**.

## Security Principles

### Data at Rest
- Database stored in the user's app data directory with OS-level permissions.
- Phase 2 will add optional ChaCha20-Poly1305 encryption (via the `age` crate).

### Data in Transit
- No data leaves the machine by default.
- Ollama communication is localhost-only (`http://127.0.0.1:11434`).
- HuggingFace Hub model download uses HTTPS (one-time, on first launch).

### Dependencies
- All Rust dependencies are audited via `cargo audit` in CI.
- Frontend dependencies are minimal and pinned via `bun.lock`.
- No third-party analytics, tracking, or error reporting SDKs.

### Build Integrity
- Builds are reproducible via GitHub Actions CI/CD.
- Release artifacts are generated in CI, not on developer machines.
- All PRs require CI to pass (tests + clippy + type checking).

## Threat Model

| Threat | Mitigation |
| ------ | ---------- |
| Data exfiltration via network | No outbound connections (enforced in code) |
| Malicious dependency | `cargo audit` in CI, minimal dependency surface |
| Local file access by other apps | OS-level file permissions, future encryption |
| Memory-resident secrets | Rust ownership model, no GC-based leaks |
| Supply chain attack | Pinned dependencies, lockfile integrity checks |

## Acknowledgments

We appreciate the security research community. Responsible disclosures will be credited in release notes (with your permission).
