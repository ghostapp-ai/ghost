---
title: Agent Engine
description: Ghost's ReAct agent with native tool calling, safety tiers, and conversation memory.
---

Ghost includes a full agent engine that can reason, plan, and execute multi-step tasks using local AI models.

## ReAct Loop

The agent follows a **Reason + Act** cycle:

```
User request
    ↓
LLM reasons about the task
    ↓
LLM selects a tool to call
    ↓
Tool executes and returns results
    ↓
LLM reasons about results
    ↓
(repeat up to 10 iterations)
    ↓
Final answer to user
```

## Built-in Tools

| Tool | Description | Safety Tier |
|------|------------|-------------|
| `search_files` | Hybrid search across indexed documents | Safe |
| `read_file` | Read file contents | Moderate |
| `list_directory` | List files in a directory | Safe |
| `get_system_info` | Hardware and platform info | Safe |
| `get_index_stats` | Database statistics | Safe |
| `web_search` | Web search via MCP server | Moderate |

Plus **unlimited external tools** via connected MCP servers.

## Safety System

Ghost uses a 3-tier risk classification:

### Safe (Auto-approved)
- Read-only operations
- Search, list, stats queries
- No user confirmation needed

### Moderate (Logged)
- File reads, especially sensitive paths
- External API calls via MCP
- Logged for transparency

### Dangerous (Requires Approval)
- File writes, deletes, or moves
- Command execution
- System modifications
- **Always asks for user confirmation**

## Conversation Memory

All conversations are persisted in SQLite:

- Full message history with timestamps
- FTS5 search across past conversations
- CASCADE deletes for clean conversation management
- Conversation summaries for context continuity

## Model Selection

The agent automatically selects the best Qwen3 model for your hardware:

```
RAM → Model Tier
━━━━━━━━━━━━━━━━
2 GB  → Qwen3-0.6B
4 GB  → Qwen3-1.7B
6 GB  → Qwen3-4B
10 GB → Qwen3-8B
18 GB → Qwen3-14B
36 GB → Qwen3-32B
```

All models support native tool calling — no prompt injection or regex parsing needed.
