//! # IronRoot MCP Example Server
//!
//! This binary is the entry point for the example MCP (Model Context Protocol)
//! server bundled with the IronRoot framework.
//!
//! ## What is MCP?
//!
//! The Model Context Protocol is an open standard that allows AI agents to
//! communicate with external tools and data sources in a structured, type-safe
//! way. An MCP server exposes *resources* and *tools* that agents can invoke.
//!
//! ## Architecture
//!
//! This server uses **stdio transport** — the agent launches the process and
//! communicates over stdin / stdout using newline-delimited JSON messages.
//!
//! ```text
//! AI Agent
//!   │  JSON-RPC (stdio)
//!   ▼
//! ironroot-mcp-example
//!   │
//!   ├── resources: workspace metadata, crate list
//!   └── tools: scaffold crate, run tests, …
//! ```
//!
//! ## TODO
//!
//! - Implement the MCP handshake (`initialize` / `initialized`).
//! - Register `resources/list` handler.
//! - Register `tools/call` handler.
//! - Add structured logging (e.g. via `tracing`).

use std::io::{self, BufRead, Write};

fn main() {
    eprintln!("[ironroot-mcp] IronRoot MCP example server starting (stdio transport)");

    // TODO: Replace this read-loop with a full MCP protocol implementation.
    //
    // The MCP protocol uses newline-delimited JSON-RPC 2.0 messages.
    // Each line read from stdin is a complete JSON-RPC request; each line
    // written to stdout is a JSON-RPC response.
    //
    // See https://spec.modelcontextprotocol.io for the full specification.

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[ironroot-mcp] error reading stdin: {e}");
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        eprintln!("[ironroot-mcp] received: {line}");

        // TODO: Parse as JSON-RPC, route to appropriate handler, and write a
        // proper JSON-RPC response.
        let response = format!(
            r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":-32601,"message":"Method not implemented yet. See mcp/README.md for the roadmap."}}}}"#
        );

        writeln!(out, "{response}").unwrap_or_else(|e| {
            eprintln!("[ironroot-mcp] error writing stdout: {e}");
        });
        out.flush().unwrap_or_default();
    }

    eprintln!("[ironroot-mcp] server shutting down");
}
