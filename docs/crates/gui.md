# `ironroot-gui`

*(Planned)* Desktop UI integration for IronRoot.

## Scope

When complete, `ironroot-gui` will provide trait abstractions and adapters
for two complementary GUI stacks:

- **[Tauri](https://tauri.app)** — web-tech UI inside a native window. Best
  for apps that want to reuse a React/Angular/Vue frontend (see
  [`ironroot`](new.md) for a bootstrap).
- **[egui](https://docs.rs/egui)** — pure-Rust immediate-mode GUI. Best for
  internal tools and apps where shipping a JS toolchain is overkill.

Selection will be via Cargo features:

```toml
[dependencies]
ironroot-gui = { version = "0.2", features = ["egui"] }
# or
ironroot-gui = { version = "0.2", features = ["tauri"] }
```

The two features will be mutually exclusive at compile time.

## Status

The crate is a placeholder today. Real work begins in **Phase 3** of the
[roadmap](../roadmap.md).

If you need a GUI today, the [`ironroot`](new.md) tool can scaffold a
client/server project with either Tauri or egui directly — without going
through `ironroot-gui`.
