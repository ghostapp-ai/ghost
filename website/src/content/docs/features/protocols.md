---
title: Protocol Hub
description: Ghost speaks MCP, A2A, AG-UI, A2UI, and WebMCP — connecting you to the entire AI ecosystem.
---

Ghost is a **Universal Protocol Hub** — it implements five open standards for AI agent interoperability.

## Protocol Stack

```
┌─────────────────────────────────────┐
│  WebMCP — Browser tool contracts    │  Phase 2.5
├─────────────────────────────────────┤
│  A2A — Multi-agent coordination     │  Phase 2
├─────────────────────────────────────┤
│  A2UI — Generative UI (JSON→React)  │  ✅ Complete
├─────────────────────────────────────┤
│  AG-UI — Agent↔User streaming       │  ✅ Complete
├─────────────────────────────────────┤
│  MCP — Model Context Protocol       │  ✅ Complete
└─────────────────────────────────────┘
```

## MCP (Model Context Protocol)

**Status**: ✅ Server + Client complete

### Ghost as MCP Server
Ghost exposes its tools (search, index, stats) to external AI assistants:

- Works with Claude Desktop, VS Code Copilot, Cursor, and any MCP client
- HTTP Streamable transport on port 6774
- Tool schemas auto-generated via `rmcp` `#[tool]` macro

### Ghost as MCP Client
Ghost connects to 10,000+ external MCP servers:

- Filesystem, GitHub, databases, APIs
- Stdio + HTTP transports
- Configurable from Settings panel

## AG-UI (Agent-User Interaction Protocol)

**Status**: ✅ Complete

CopilotKit's open standard for real-time agent↔user communication:

- ~16 event types (TEXT_MESSAGE_START, TOOL_CALL, AGENT_STATE_UPDATE, etc.)
- Bidirectional streaming via Tauri IPC (desktop) and SSE (external)
- Human-in-the-loop for dangerous operations
- Powers all chat streaming in Ghost

## A2UI (Generative UI)

**Status**: ✅ Complete

Google's declarative UI specification rendered natively in React:

- 17+ component types: Text, Button, TextField, Card, Row, Column, Chip, etc.
- Two-way data binding via JSON Pointers (RFC 6901)
- Adjacency list → tree resolution on frontend
- Transported via AG-UI CUSTOM events

## A2A (Agent-to-Agent)

**Status**: Phase 2

Google + Linux Foundation standard for multi-agent coordination:

- Agent Cards at `/.well-known/agent.json`
- JSON-RPC 2.0 task delegation
- SSE for streaming progress
- Ghost can delegate tasks to specialized agents

## WebMCP (Web Agent Protocol)

**Status**: Phase 2.5

W3C incubation for browser-based tool contracts:

- `navigator.modelContext` browser API
- Structured web interactions without scraping
- Browser extension bridge between Ghost and web tools
