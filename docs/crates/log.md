# `ironroot-log`

Default logging facade for IronRoot — opinionated, batteries-included setup
built on [`tracing`](https://docs.rs/tracing).

## What's in it

- **Size-based file rotation** (default 10 MB, 5 retained files).
- **Optional syslog export** via the `syslog` feature (Unix socket or remote
  UDP).
- **stdout passthrough** for development and container deployments.
- Builder API (`LogConfig`) and a one-call `init_default(app_name)` helper.

## Cargo

```toml
[dependencies]
ironroot-log = "0.2"

# With syslog export:
ironroot-log = { version = "0.2", features = ["syslog"] }
```

## Quick start

```rust
fn main() {
    let _guard = ironroot_log::init_default("my-app")
        .expect("logger init");
    tracing::info!("ready");
    // ... rest of the program ...
}
```

Hold on to the returned `LogGuard` for the whole program lifetime —
dropping it flushes buffered log lines.

## Custom configuration

```rust
use ironroot_log::LogConfig;

let _guard = LogConfig::new("my-app")
    .with_directory("/var/log/my-app")
    .with_max_bytes(50 * 1024 * 1024)   // 50 MB per file
    .with_keep_files(10)                // keep 10 rotated files
    .with_env_filter("info,my_app=debug")
    .init()?;
```

## Enabling syslog

```rust
# #[cfg(feature = "syslog")]
# {
use ironroot_log::{LogConfig, SyslogTarget};

let _guard = LogConfig::new("my-app")
    .with_directory("./logs")
    .with_syslog(SyslogTarget::LocalUnix)
    .init()?;
// or:
// .with_syslog(SyslogTarget::Udp { address: "logs.internal:514".into() })
# }
```

## Threat model & notes

- File writes are non-blocking via `tracing-appender`'s background worker.
  Drop the `LogGuard` cleanly at shutdown to flush.
- Syslog delivery is best-effort — failures are silently dropped so logging
  never panics the program.
- The on-disk format is plain text (no ANSI colours in file output) to keep
  rotated files greppable.

## Constants

| Constant | Value | Meaning |
|---|---|---|
| `DEFAULT_MAX_BYTES` | `10 * 1024 * 1024` | 10 MB rotation threshold |
| `DEFAULT_KEEP_FILES` | `5` | retained historical files |

## Error type

```rust
pub enum LogError {
    CreateDir { path: PathBuf, source: io::Error },
    SubscriberAlreadySet,
    #[cfg(feature = "syslog")]
    Syslog(String),
}
```
