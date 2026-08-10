//! # ironroot-log
//!
//! Default logging facade for the IronRoot framework.
//!
//! Provides an opinionated, batteries-included logging setup built on top of
//! [`tracing`] with:
//!
//! - **Size-based file rotation** — log files are rotated when they reach a
//!   configurable byte limit (default 10 MB) and a configurable number of
//!   historical files are retained.
//! - **Optional syslog export** (behind the `syslog` feature flag) — emits
//!   the same events to a local or remote syslog daemon in parallel with the
//!   file sink.
//! - **stdout passthrough** — human-readable formatting on standard output by
//!   default, useful for development and containerised deployments.
//!
//! ## Quick start
//!
//! ```no_run
//! use ironroot_log::LogConfig;
//!
//! let _guard = LogConfig::new("my-app")
//!     .with_directory("./logs")
//!     .init()
//!     .expect("logger init");
//!
//! tracing::info!("hello, world");
//! ```
//!
//! Hold on to the returned [`LogGuard`] for the lifetime of the program —
//! dropping it flushes buffered log lines.
//!
//! ## Enabling syslog
//!
//! Add the feature to your `Cargo.toml`:
//!
//! ```toml
//! ironroot-log = { version = "0.2", features = ["syslog"] }
//! ```
//!
//! Then opt in at runtime:
//!
//! ```no_run
//! # #[cfg(feature = "syslog")]
//! # {
//! use ironroot_log::{LogConfig, SyslogTarget};
//!
//! let _guard = LogConfig::new("my-app")
//!     .with_directory("./logs")
//!     .with_syslog(SyslogTarget::LocalUnix)
//!     .init()
//!     .expect("logger init");
//! # }
//! ```

use std::io;
use std::path::{Path, PathBuf};

use file_rotate::{ContentLimit, FileRotate, compression::Compression, suffix::AppendCount};
use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Default rotation threshold: 10 MB.
pub const DEFAULT_MAX_BYTES: usize = 10 * 1024 * 1024;

/// Default number of rotated files retained.
pub const DEFAULT_KEEP_FILES: usize = 5;

/// Errors that can occur during logger initialisation.
#[derive(Debug, Error)]
pub enum LogError {
    /// The log directory could not be created.
    #[error("failed to create log directory {path:?}: {source}")]
    CreateDir {
        /// Directory that failed to be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Another global tracing subscriber was already installed.
    #[error("a global tracing subscriber is already set")]
    SubscriberAlreadySet,

    /// Syslog initialisation failed.
    #[cfg(feature = "syslog")]
    #[error("failed to connect to syslog: {0}")]
    Syslog(String),
}

/// Where to send syslog events.
#[cfg(feature = "syslog")]
#[derive(Debug, Clone)]
pub enum SyslogTarget {
    /// Connect to the local syslog daemon via the standard Unix socket
    /// (`/dev/log` on Linux, `/var/run/syslog` on macOS).
    LocalUnix,
    /// Send to a remote syslog daemon over UDP.
    Udp {
        /// `host:port` of the remote daemon.
        address: String,
    },
}

/// Builder-style configuration for the IronRoot logger.
#[derive(Debug, Clone)]
pub struct LogConfig {
    #[cfg_attr(not(feature = "syslog"), allow(dead_code))]
    app_name: String,
    directory: PathBuf,
    file_name: String,
    max_bytes: usize,
    keep_files: usize,
    env_filter: Option<String>,
    stdout: bool,
    #[cfg(feature = "syslog")]
    syslog: Option<SyslogTarget>,
}

impl LogConfig {
    /// Create a new configuration for the given application name. The name is
    /// used as the syslog process tag and as the default log file stem.
    pub fn new(app_name: impl Into<String>) -> Self {
        let name = app_name.into();
        let file_name = format!("{name}.log");
        Self {
            app_name: name,
            directory: PathBuf::from("./logs"),
            file_name,
            max_bytes: DEFAULT_MAX_BYTES,
            keep_files: DEFAULT_KEEP_FILES,
            env_filter: None,
            stdout: true,
            #[cfg(feature = "syslog")]
            syslog: None,
        }
    }

    /// Set the directory where log files are written. Defaults to `./logs`.
    pub fn with_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.directory = dir.into();
        self
    }

    /// Set the log file name (just the leaf, not the directory). Defaults to
    /// `<app_name>.log`.
    pub fn with_file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = name.into();
        self
    }

    /// Set the rotation threshold in bytes. Defaults to [`DEFAULT_MAX_BYTES`]
    /// (10 MB).
    pub fn with_max_bytes(mut self, bytes: usize) -> Self {
        self.max_bytes = bytes;
        self
    }

    /// Set the number of rotated historical files to retain. Older files are
    /// deleted. Defaults to [`DEFAULT_KEEP_FILES`].
    pub fn with_keep_files(mut self, n: usize) -> Self {
        self.keep_files = n;
        self
    }

    /// Override the env-filter directive (e.g. `"info,my_crate=debug"`).
    /// Defaults to `RUST_LOG` or `info` if unset.
    pub fn with_env_filter(mut self, directive: impl Into<String>) -> Self {
        self.env_filter = Some(directive.into());
        self
    }

    /// Enable or disable stdout output. Defaults to enabled.
    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }

    /// Enable syslog export to the given target.
    #[cfg(feature = "syslog")]
    pub fn with_syslog(mut self, target: SyslogTarget) -> Self {
        self.syslog = Some(target);
        self
    }

    /// Install the configured logger as the global `tracing` subscriber.
    ///
    /// Returns a [`LogGuard`] that **must** be kept alive for the duration of
    /// the program — dropping it flushes any buffered log lines.
    pub fn init(self) -> Result<LogGuard, LogError> {
        std::fs::create_dir_all(&self.directory).map_err(|source| LogError::CreateDir {
            path: self.directory.clone(),
            source,
        })?;

        let log_path = self.directory.join(&self.file_name);
        let rotator = FileRotate::new(
            log_path,
            AppendCount::new(self.keep_files),
            ContentLimit::Bytes(self.max_bytes),
            Compression::None,
            #[cfg(unix)]
            None,
        );

        let (file_writer, file_guard) = tracing_appender::non_blocking(rotator);

        let env_filter = match &self.env_filter {
            Some(d) => EnvFilter::try_new(d).unwrap_or_else(|_| EnvFilter::new("info")),
            None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        };

        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_writer(WriterFactory(file_writer));

        // Optional layers are wrapped in `Option<_>` so all four combinations
        // (stdout x syslog) compose into a single subscriber type.
        let stdout_layer = if self.stdout {
            Some(fmt::layer().with_writer(std::io::stdout))
        } else {
            None
        };

        #[cfg(feature = "syslog")]
        let syslog_layer = match self.syslog.clone() {
            Some(target) => {
                let writer = build_syslog_writer(&self.app_name, target)?;
                Some(fmt::layer().with_ansi(false).with_writer(writer))
            }
            None => None,
        };

        let registry = tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer);

        #[cfg(feature = "syslog")]
        let registry = registry.with(syslog_layer);

        registry
            .try_init()
            .map_err(|_| LogError::SubscriberAlreadySet)?;

        Ok(LogGuard { _file: file_guard })
    }
}

/// RAII guard returned by [`LogConfig::init`]. Drop it to flush buffered logs.
#[must_use = "dropping the LogGuard flushes pending log lines; keep it alive for the program lifetime"]
pub struct LogGuard {
    _file: WorkerGuard,
}

// `tracing-appender`'s NonBlocking implements `Write` but not directly
// `MakeWriter`; wrap it so each event gets a fresh handle.
struct WriterFactory(tracing_appender::non_blocking::NonBlocking);

impl<'a> MakeWriter<'a> for WriterFactory {
    type Writer = tracing_appender::non_blocking::NonBlocking;

    fn make_writer(&'a self) -> Self::Writer {
        self.0.clone()
    }
}

// --- syslog support --------------------------------------------------------

#[cfg(feature = "syslog")]
fn build_syslog_writer(app_name: &str, target: SyslogTarget) -> Result<SyslogWriter, LogError> {
    use std::sync::Mutex;
    use syslog::{Facility, Formatter3164};

    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: app_name.to_string(),
        pid: std::process::id(),
    };

    let logger = match target {
        SyslogTarget::LocalUnix => {
            syslog::unix(formatter).map_err(|e| LogError::Syslog(e.to_string()))?
        }
        SyslogTarget::Udp { address } => syslog::udp(formatter, "0.0.0.0:0", &address)
            .map_err(|e| LogError::Syslog(e.to_string()))?,
    };

    Ok(SyslogWriter(std::sync::Arc::new(Mutex::new(logger))))
}

#[cfg(feature = "syslog")]
#[derive(Clone)]
struct SyslogWriter(
    std::sync::Arc<std::sync::Mutex<syslog::Logger<syslog::LoggerBackend, syslog::Formatter3164>>>,
);

#[cfg(feature = "syslog")]
impl io::Write for SyslogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let line = String::from_utf8_lossy(buf);
        // syslog records are line-oriented; trim the trailing newline.
        let trimmed = line.trim_end_matches('\n');
        if let Ok(mut logger) = self.0.lock() {
            // best-effort: ignore syslog delivery errors so logging never panics
            let _ = logger.info(trimmed);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "syslog")]
impl<'a> MakeWriter<'a> for SyslogWriter {
    type Writer = SyslogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

// --- convenience -----------------------------------------------------------

/// One-call default initialiser: writes to `./logs/<app_name>.log` with 10MB
/// rotation, retains 5 files, and mirrors to stdout.
pub fn init_default(app_name: &str) -> Result<LogGuard, LogError> {
    LogConfig::new(app_name).init()
}

/// Return the canonical path where logs would be written for the given
/// directory and app name. Useful for diagnostics.
pub fn log_file_path(directory: &Path, app_name: &str) -> PathBuf {
    directory.join(format!("{app_name}.log"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_sensible() {
        let cfg = LogConfig::new("demo");
        assert_eq!(cfg.app_name, "demo");
        assert_eq!(cfg.file_name, "demo.log");
        assert_eq!(cfg.max_bytes, DEFAULT_MAX_BYTES);
        assert_eq!(cfg.keep_files, DEFAULT_KEEP_FILES);
        assert!(cfg.stdout);
    }

    #[test]
    fn builder_overrides_apply() {
        let cfg = LogConfig::new("demo")
            .with_directory("/tmp/x")
            .with_file_name("custom.log")
            .with_max_bytes(1024)
            .with_keep_files(2)
            .with_stdout(false)
            .with_env_filter("debug");
        assert_eq!(cfg.directory, PathBuf::from("/tmp/x"));
        assert_eq!(cfg.file_name, "custom.log");
        assert_eq!(cfg.max_bytes, 1024);
        assert_eq!(cfg.keep_files, 2);
        assert!(!cfg.stdout);
        assert_eq!(cfg.env_filter.as_deref(), Some("debug"));
    }

    #[test]
    fn log_file_path_joins_correctly() {
        let p = log_file_path(Path::new("/var/log"), "svc");
        assert_eq!(p, PathBuf::from("/var/log/svc.log"));
    }

    #[test]
    fn init_creates_directory_and_rotates() {
        // Run init in a fresh temp dir; we don't actually install the global
        // subscriber here (other tests might collide) — just exercise the
        // directory-creation + rotator wiring.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("nested/logs");
        assert!(!dir.exists());

        // Manually replicate the directory-creation step and rotator creation
        // so this test does not race with parallel subscriber installs.
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("svc.log");
        let mut rotator = file_rotate::FileRotate::new(
            &path,
            file_rotate::suffix::AppendCount::new(2),
            file_rotate::ContentLimit::Bytes(64),
            file_rotate::compression::Compression::None,
            #[cfg(unix)]
            None,
        );
        use std::io::Write as _;
        for _ in 0..10 {
            writeln!(rotator, "hello world line that is reasonably long").unwrap();
        }
        rotator.flush().unwrap();

        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert!(
            entries.len() >= 2,
            "expected rotated files, got {}",
            entries.len()
        );
    }
}
