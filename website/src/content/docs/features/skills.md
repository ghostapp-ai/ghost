---
title: Skills System
description: Ghost's SKILL.md-based plugin system for extensible agent capabilities.
---

Ghost uses a **Skills.md** format inspired by [OpenClaw](https://github.com/nicepkg/OpenClaw) for defining reusable agent capabilities.

## What Is a Skill?

A skill is a Markdown file with YAML frontmatter that teaches Ghost how to perform a specific task. Skills can:

- Define tool schemas that the agent can call
- Specify triggers (keywords, patterns) for automatic activation
- Include instructions, examples, and constraints
- Reference MCP servers for external tool access

## SKILL.md Format

```markdown
---
name: file-organizer
version: 1.0.0
description: Organizes files by type, date, or project
triggers:
  - organize files
  - sort files
  - clean up folder
tools:
  - name: move_file
    description: Move a file to a new location
    parameters:
      source: { type: string, description: "Source file path" }
      destination: { type: string, description: "Destination path" }
  - name: create_folder
    description: Create a new folder
    parameters:
      path: { type: string, description: "Folder path to create" }
safety: moderate
---

# File Organizer

You are a file organization assistant. When the user asks to
organize files, follow these steps:

1. List the files in the target directory
2. Classify each file by type (documents, images, code, etc.)
3. Propose an organization plan
4. Ask for confirmation before moving files
5. Execute the moves and report results

## Rules
- Never delete files — only move them
- Always ask before moving to a different drive
- Preserve original filenames
```

## Skill Registry

Ghost maintains a `SkillRegistry` that:

1. Scans for `.skill.md` files in configured directories
2. Parses YAML frontmatter for metadata and tool schemas
3. Indexes triggers for fast matching
4. Registers tools with the agent engine

## Trigger Matching

When a user message arrives, the skill registry checks:

1. **Exact trigger match**: "organize files" → file-organizer skill
2. **Partial match**: "can you organize my files?" → fuzzy matching
3. **Skill name**: "@file-organizer" → direct invocation

## Built-in Skills vs. Custom

Ghost ships with essential built-in skills. Users can create custom skills by placing `.skill.md` files in their skill directory.

### Future: Skills Marketplace

Phase 3 includes a Skills Marketplace for:
- Community-contributed skills
- Verified/audited skills
- Monetization for skill authors
- One-click install from Ghost UI
