use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::interpreter::TraceEntry;

#[derive(Clone)]
pub(super) struct TraceLogger {
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl TraceLogger {
    pub(super) fn open(path: &PathBuf) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    pub(super) fn log_entry(&self, step: usize, entry: &TraceEntry) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "[Step {:3}] {}", step, entry);
            let _ = writer.flush();
        }
    }

    pub(super) fn log_header(
        &self,
        entry: &str,
        goal: &str,
        input_source_kind: &str,
        game_name: Option<&str>,
    ) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "=== MCG Trace Log ===");
            let _ = writeln!(
                writer,
                "Version: {} (cgdsl-engine)",
                env!("CARGO_PKG_VERSION")
            );
            let _ = writeln!(writer, "Started: {}", format_timestamp());
            if let Some(name) = game_name {
                let _ = writeln!(writer, "Game: {}", name);
            }
            let _ = writeln!(writer, "Entry: {}", entry);
            let _ = writeln!(writer, "Goal: {}", goal);
            let _ = writeln!(writer, "Input source: {}", input_source_kind);
            let _ = writeln!(writer, "====================");
            let _ = writer.flush();
        }
    }

    pub(super) fn log_footer(&self, status: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "=== {} ===", status);
            let _ = writer.flush();
        }
    }

    pub(super) fn log_panic(&self, msg: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "=== Panic: {} ===", msg);
            let _ = writer.flush();
        }
    }

    pub(super) fn flush(&self) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.flush();
        }
    }
}

/// Resolve the trace-log path.
///
/// Precedence: an explicit path from `RunOptions::with_log_path` wins; otherwise
/// the `MCG_TRACE_LOG` env var is consulted (`""`/`"off"`/`"none"` disable);
/// when neither is set, **no trace file is written** — the library never
/// creates files in the working directory on its own.
pub(super) fn resolve_log_path(explicit: Option<&std::path::Path>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return Some(path.to_path_buf());
    }
    match std::env::var("MCG_TRACE_LOG") {
        Ok(val) => {
            let val = val.trim();
            if val.is_empty() || val.eq_ignore_ascii_case("off") || val.eq_ignore_ascii_case("none")
            {
                None
            } else {
                Some(PathBuf::from(val))
            }
        }
        Err(_) => None,
    }
}

/// UTC timestamp `YYYY-MM-DD HH:MM:SS` without external dependencies.
///
/// Uses Howard Hinnant's public-domain `civil_from_days` algorithm for the
/// date portion; the input is assumed to be within the supported range
/// (approximately years 0001..9999, well beyond any real run).
fn format_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        y,
        m,
        d,
        rem / 3_600,
        (rem % 3_600) / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serializes the tests that mutate `MCG_TRACE_LOG` — process env is
    /// shared across the parallel test threads, so racing mutations would
    /// make the assertions flaky.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn timestamp_is_iso_utc() {
        let ts = format_timestamp();
        assert!(
            ts.len() == 23 && ts.ends_with(" UTC"),
            "unexpected timestamp: {ts}"
        );
        assert!(
            ts.chars()
                .take(10)
                .enumerate()
                .all(|(i, c)| (i == 4 || i == 7) && c == '-' || c.is_ascii_digit()),
            "date part must be YYYY-MM-DD, got: {ts}"
        );
        assert!(
            ts.chars()
                .skip(11)
                .take(8)
                .enumerate()
                .all(|(i, c)| { (i == 2 || i == 5) && c == ':' || c.is_ascii_digit() }),
            "time part must be HH:MM:SS, got: {ts}"
        );
    }

    #[test]
    fn resolve_log_path_prefers_explicit_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("MCG_TRACE_LOG");
        let explicit = std::path::PathBuf::from("explicit.log");
        assert_eq!(
            resolve_log_path(Some(&explicit)),
            Some(explicit.clone()),
            "explicit path must win over the env var"
        );
    }

    #[test]
    fn resolve_log_path_defaults_to_off() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("MCG_TRACE_LOG");
        assert_eq!(resolve_log_path(None), None, "no env, no option: no file");
    }

    #[test]
    fn resolve_log_path_honors_env_var() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("MCG_TRACE_LOG", "env-trace.log");
        assert_eq!(resolve_log_path(None), Some(PathBuf::from("env-trace.log")));
        std::env::set_var("MCG_TRACE_LOG", "off");
        assert_eq!(resolve_log_path(None), None, "\"off\" disables");
        std::env::set_var("MCG_TRACE_LOG", "NONE");
        assert_eq!(
            resolve_log_path(None),
            None,
            "\"NONE\" disables (case-insensitive)"
        );
        std::env::remove_var("MCG_TRACE_LOG");
    }
}
