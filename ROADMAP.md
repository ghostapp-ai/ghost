# Ghost — Roadmap

## M0: Monetization Foundation (Month 0–1)

**Goal:** First paying customer.

### Stripe Integration
- [ ] Add `stripe` crate or HTTP client for Stripe API
- [ ] Implement Checkout Session creation for Pro ($8/mo) and Teams ($15/user/mo)
- [ ] Customer Portal link for subscription management
- [ ] Webhook handler for `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`
- [ ] Store subscription status in local SQLite (synced on app start)

### License Key Validation
- [ ] License key generation service (Cloudflare Worker or Stripe metadata)
- [ ] Validation endpoint: key → plan tier + expiry
- [ ] `ghost-pro` crate: check license on startup, cache result locally
- [ ] Graceful degradation: Pro features disabled without valid key, no crashes
- [ ] Offline grace period: 7 days without validation before locking Pro features

### Feature Gating
- [ ] Audit all Pro features in `ghost-pro` crate
- [ ] Gate semantic search behind Pro
- [ ] Gate advanced MCP catalog (30+ curated) behind Pro
- [ ] Gate team features behind Teams tier
- [ ] Free tier: local AI + FTS5 + basic tools (fully functional, not crippled)

### Landing Page
- [ ] Deploy ghost.quirozai.com (or ghost-app.dev)
- [ ] Hero: "Your AI assistant that never phones home"
- [ ] Feature comparison table (Free vs Pro vs Teams)
- [ ] Download links per platform
- [ ] Stripe Checkout integration

**Revenue target M0:** $0 → first 10 paying users, ~$80 MRR

---

## M1: Distribution & Growth (Month 2–3)

**Goal:** Reach 500 downloads, 50 Pro users.

### App Store Distribution
- [ ] macOS: Code signing + notarization + DMG packaging
- [ ] Windows: Code signing + Microsoft Store submission
- [ ] Linux: AppImage + Snap/Flatpak
- [ ] Auto-updater via Tauri v2 updater plugin

### Onboarding
- [ ] First-run wizard: import files, set up file watcher directories
- [ ] "Quick Start" with pre-configured MCP servers
- [ ] Interactive tutorial: first search, first agent task

### SEO & Content
- [ ] Comparison pages: "Ghost vs Raycast", "Ghost vs Recall", "Ghost vs Rewind"
- [ ] Blog posts on privacy-first AI workflow
- [ ] Product Hunt launch

### Polish
- [ ] Crash reporting (opt-in, local-first)
- [ ] Performance benchmarks page (FTS5 latency, embedding speed)
- [ ] Keyboard shortcuts customization

**Revenue target M1:** $80 → $400 MRR (50 Pro users)

---

## M2: Teams & Enterprise (Month 4–6)

**Goal:** Land first team customer, reach $2K MRR.

### Teams Features
- [ ] Shared knowledge base sync (encrypted, peer-to-peer or self-hosted relay)
- [ ] Team MCP server registry (admin curates, members consume)
- [ ] Admin dashboard: user management, usage analytics
- [ ] SSO integration (SAML/OIDC)
- [ ] Audit logs for compliance

### Agent Improvements
- [ ] Multi-model support: add Llama 3, Phi-3, Mistral GGUF variants
- [ ] Agent memory: persistent context across sessions
- [ ] Skill marketplace: community-contributed agent skills
- [ ] A2A federation: Ghost instances collaborating on tasks

### Infrastructure
- [ ] Telemetry dashboard (opt-in, aggregated, no PII)
- [ ] Automated release pipeline with changelog generation
- [ ] Beta channel for early adopters

**Revenue target M2:** $400 → $2,000 MRR (50 Pro + 10 Teams seats)

---

## Revenue Trajectory

| Month | Pro Users | Teams Seats | MRR |
|---|---|---|---|
| M0 | 10 | 0 | $80 |
| M1 | 50 | 0 | $400 |
| M2 | 50 | 10 | $550 |
| M3 | 100 | 30 | $1,250 |
| M6 | 250 | 100 | $3,500 |
| M12 | 1,000 | 500 | $15,500 |

**Break-even:** ~$500 MRR covers infrastructure costs. Target by M2.
