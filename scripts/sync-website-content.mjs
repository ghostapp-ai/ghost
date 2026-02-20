#!/usr/bin/env node

/**
 * sync-website-content.mjs
 *
 * Deterministic content synchronization script (no AI required).
 * Copies and adapts source-of-truth files into the Astro Starlight website.
 *
 * Run: node scripts/sync-website-content.mjs
 *
 * This script handles the "boring" deterministic sync:
 * - CHANGELOG.md → website changelog page
 * - CONTRIBUTING.md → website contributing page
 * - SECURITY.md → website privacy page
 * - ROADMAP.md → website roadmap page
 *
 * The AI agents handle the "smart" sync:
 * - README.md sections → multiple website pages (requires understanding)
 * - CLAUDE.md architecture → multiple architecture pages (requires extraction)
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const DOCS = join(ROOT, "website", "src", "content", "docs");

// ── Helpers ──────────────────────────────────────────────────────

function read(relPath) {
  const full = join(ROOT, relPath);
  if (!existsSync(full)) {
    console.warn(`⚠ Source file not found: ${relPath}`);
    return null;
  }
  return readFileSync(full, "utf-8");
}

function write(relPath, content) {
  const full = join(DOCS, relPath);
  mkdirSync(dirname(full), { recursive: true });
  writeFileSync(full, content, "utf-8");
  console.log(`✓ Synced → website/src/content/docs/${relPath}`);
}

function wrapFrontmatter(title, description, body) {
  return `---
title: "${title}"
description: "${description}"
---

${body.trim()}
`;
}

function stripH1(content) {
  // Remove the first # heading (Starlight generates it from frontmatter)
  return content.replace(/^#\s+.+\n+/, "");
}

// ── Sync Functions ───────────────────────────────────────────────

function syncChangelog() {
  const content = read("CHANGELOG.md");
  if (!content) return;

  const body = stripH1(content);
  write(
    "reference/changelog.md",
    wrapFrontmatter(
      "Changelog",
      "Release notes and version history for Ghost Agent OS.",
      body
    )
  );
}

function syncRoadmap() {
  const content = read("ROADMAP.md");
  if (!content) return;

  const body = stripH1(content);
  write(
    "reference/roadmap.md",
    wrapFrontmatter(
      "Roadmap",
      "Development roadmap and upcoming features for Ghost Agent OS.",
      body
    )
  );
}

function syncContributing() {
  const content = read("CONTRIBUTING.md");
  if (!content) return;

  const body = stripH1(content);
  write(
    "reference/contributing.md",
    wrapFrontmatter(
      "Contributing",
      "How to contribute to Ghost — guidelines, setup, and development workflow.",
      body
    )
  );
}

function syncSecurity() {
  const content = read("SECURITY.md");
  if (!content) return;

  const body = stripH1(content);
  write(
    "reference/privacy.md",
    wrapFrontmatter(
      "Privacy & Security",
      "Ghost's privacy-first architecture — how your data stays local and secure.",
      body
    )
  );
}

// ── Version sync ─────────────────────────────────────────────────

function getVersion() {
  try {
    const cargoToml = read("src-tauri/Cargo.toml");
    if (!cargoToml) return null;
    const match = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
    return match ? match[1] : null;
  } catch {
    return null;
  }
}

// ── Main ─────────────────────────────────────────────────────────

function main() {
  console.log("🔄 Syncing website content from source files...\n");

  const version = getVersion();
  if (version) {
    console.log(`📦 Current version: ${version}\n`);
  }

  syncChangelog();
  syncRoadmap();
  syncContributing();
  syncSecurity();

  console.log("\n✅ Content sync complete!");
  console.log(
    "ℹ  README.md and CLAUDE.md require AI-assisted sync (via Claude/Copilot agents).\n"
  );
}

main();
