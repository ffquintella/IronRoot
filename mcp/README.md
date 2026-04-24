# IronRoot — MCP Integration

## What is MCP?

The **Model Context Protocol (MCP)** is an open standard developed by Anthropic that defines how AI agents communicate with external tools, data sources, and services. It provides a structured, type-safe JSON-RPC interface that any agent — Claude, GitHub Copilot, GPT-based tools — can use to interact with your application.

Full specification: <https://spec.modelcontextprotocol.io>

---

## Role in IronRoot

MCP servers in IronRoot serve as the **bridge between AI agents and the framework itself**. They allow agents to:

- Inspect the workspace structure (list crates, templates, macros).
- Generate scaffolding (new crate, new template, new macro).
- Run build and test commands and receive structured results.
- Query documentation and architecture information.

This makes IronRoot **AI-orchestrable**: an agent can understand the project layout, make targeted changes, and verify correctness — all through the MCP interface.

---

## Transport

IronRoot MCP servers use **stdio transport** by default:

1. The agent spawns the server process.
2. The agent writes JSON-RPC requests to the process's stdin.
3. The server writes JSON-RPC responses to stdout.
4. Diagnostic logs go to stderr (never mixed with the protocol stream).

An HTTP/SSE transport variant may be added in a future phase.

---

## Example Server

The `mcp/servers/example_server/` crate provides a minimal scaffold:

```
mcp/servers/example_server/
├── Cargo.toml
└── src/
    └── main.rs   ← stdio read-loop, protocol stubs
```

Build and run it with:

```bash
cargo build -p ironroot-mcp-example
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | ./target/debug/ironroot-mcp-example
```

---

## Future Integration with Agents

Planned MCP resources and tools:

| Name | Type | Description |
|---|---|---|
| `ironroot://workspace` | Resource | Workspace metadata (members, versions) |
| `ironroot://crates` | Resource | List of all framework crates |
| `scaffold/crate` | Tool | Generate a new crate from a template |
| `scaffold/template` | Tool | Generate a new application template |
| `build/run` | Tool | Run `cargo build` and return structured output |
| `test/run` | Tool | Run `cargo test` and return structured output |

These will be implemented in **Phase 4** of the roadmap.

See [docs/roadmap.md](../docs/roadmap.md) for the full plan.
