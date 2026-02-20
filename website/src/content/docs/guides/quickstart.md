---
title: Quick Start
description: Get up and running with Ghost in 2 minutes.
---

## Launch Ghost

After installation, launch Ghost from your applications menu or desktop shortcut.

## The Omnibox

Ghost's primary interface is a **unified Omnibox** — a single intelligent input that auto-detects whether you want to search or chat.

### Open the Omnibox

| Platform | Shortcut |
|----------|----------|
| Windows/Linux | `Ctrl + Space` |
| macOS | `Cmd + Space` |

### Search Mode

Type a filename, keyword, or path pattern:

```
budget.xlsx
project proposal
*.rs indexer
```

Ghost returns hybrid results combining:
- **FTS5 keyword search** (<5ms) — finds exact text matches
- **Semantic vector search** (<500ms) — finds conceptually similar content
- **RRF ranking** — combines both for the best results

### Chat Mode

Type a natural language question:

```
What files did I work on yesterday?
Summarize my project notes
Explain the search algorithm
```

Ghost auto-detects chat intent via heuristics:
- Questions starting with "what", "how", "why", "explain"
- Conversational starters like "tell me", "help me"
- Sticky mode: once in chat, stays in chat until cleared

### Toggle Modes

Click the mode indicator (🔍/💬) or use the auto-detection to switch between search and chat.

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate results |
| `Enter` | Open selected file / Send chat message |
| `Escape` | Dismiss Ghost |
| `Ctrl+D` | Toggle debug panel |
| `Ctrl+,` | Open settings |

## Settings

Access Settings via the gear icon or `Ctrl+,`:

- **General**: Launch on startup, theme preferences
- **AI Models**: Choose chat model, configure Ollama
- **Directories**: Add/remove watched folders for indexing

## Next Steps

- Learn about [Search features](/ghost/features/search/)
- Explore the [Native AI engine](/ghost/features/native-ai/)
- Set up [MCP integrations](/ghost/features/protocols/)
