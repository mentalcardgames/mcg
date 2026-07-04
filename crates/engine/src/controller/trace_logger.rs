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

    pub(super) fn log_header(&self, entry: &str, goal: &str, input_source_kind: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(writer, "=== MCG Trace Log ===");
            let _ = writeln!(writer, "Started: {}", timestamp);
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

pub(super) fn resolve_log_path() -> Option<PathBuf> {
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
        Err(_) => {
            if cfg!(test) {
                None
            } else {
                Some(PathBuf::from("mcg-trace.log"))
            }
        }
    }
}
