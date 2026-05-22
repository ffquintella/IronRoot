# `ironroot`

Interactive project bootstrapper for the IronRoot framework.

## Run it

From the workspace root:

```bash
make new
# or:
cargo run -p ironroot
```

## What it asks

1. **Project name** — kebab/snake case, 3+ chars
2. **Project kind** — CLI tool / web application / client-server
3. **GUI toolkit** *(client-server only)* — Tauri / egui
4. **Frontend framework** *(when a web UI is involved)* — React / Angular
5. **Database** — none / SQLite / PostgreSQL / MySQL
6. **Target directory** — defaults to `./<name>`

## What it generates

| File | Contents |
|---|---|
| `Cargo.toml` | Single crate for CLI/webapp, workspace with `server/` + `client/` for client-server |
| `Makefile` | `build` / `test` / `run` / `fmt` / `lint` / `check` / `clean` (plus `frontend-*`) |
| `src/main.rs` | axum server, egui/Tauri client, or CLI scaffold — with rotating file logger pre-wired |
| `tests/smoke.rs` | Integration test stub |
| `AGENTS.md` | Project-specific AI-assistant conventions |
| `README.md`, `.gitignore`, `.env.example`, `rust-toolchain.toml` | Standard project metadata |
| `frontend/` *(if applicable)* | Minimal React+Vite+TS or Angular skeleton |

Generated projects use the same **10 MB rotating file logger** defaults
that [`ironroot-log`](log.md) ships — out of the box `logs/<app>.log` will
populate as soon as you run the binary.

## Layouts

### CLI tool

```
my-cli/
├── Cargo.toml
├── src/main.rs
└── tests/smoke.rs
```

### Web application

```
my-web/
├── Cargo.toml
├── src/main.rs              # axum server
├── tests/smoke.rs
└── frontend/                # only if a frontend was selected
    ├── package.json
    └── src/...
```

### Client / server

```
my-app/
├── Cargo.toml               # workspace
├── server/                  # axum
│   ├── Cargo.toml
│   └── src/main.rs
├── client/                  # tauri or egui
│   ├── Cargo.toml
│   └── src/main.rs
└── frontend/                # only when Tauri was selected
```

## Validation

- Project name must be ASCII alphanumeric + `-` / `_`, not starting with a
  digit.
- Target directory must be empty (or non-existent).
- Existing IronRoot workspace files are never overwritten.

## See also

- The root [`Makefile`](https://github.com/ffquintella/IronRoot/blob/main/Makefile)
  exposes `make new` as a shortcut.
- [`ironroot-log`](log.md) — the rotating logger pre-wired into generated projects.
