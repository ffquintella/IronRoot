# `ironroot-web`

*(Planned)* HTTP layer integration for IronRoot.

## Scope

When complete, `ironroot-web` will provide:

- `Router` / `Middleware` / `Handler` trait abstractions independent of any
  specific HTTP framework.
- Adapters for Axum (first target) and Actix-web (follow-up).
- `HttpContext` — a typed request/response container that exposes:
  - request body decoding (JSON, form, multipart)
  - response builders with sensible defaults
  - error → status-code mapping
- Integration with [`ironroot-auth`](auth.md) for session/token middleware.
- Integration with [`ironroot-log`](log.md) for structured request logging.

## Cargo

```toml
[dependencies]
ironroot-web = "0.2"  # not yet implemented; tracking issue forthcoming
```

## Status

The crate is empty today — the module-level doc comment in `src/lib.rs`
exists as a placeholder so the workspace builds. Real abstractions will
land in **Phase 3** of the [roadmap](../roadmap.md).

If you need HTTP today, use [`axum`](https://docs.rs/axum) directly inside
your application crate; the bootstrap tool ([`ironroot-new`](new.md))
generates a working axum scaffold.
