use crate::core::{LogLevel, SourceId};
use crate::processing::ProcessedEvent;
use crate::sinks::EventSink;

use chrono::{DateTime, SecondsFormat, Utc};

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

pub struct FileSink {
    w: Mutex<BufWriter<File>>,
}

impl FileSink {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let f = File::create(path)?;
        Ok(Self {
            w: Mutex::new(BufWriter::new(f)),
        })
    }
}

impl EventSink for FileSink {
    fn emit(&self, event: &ProcessedEvent) {
        let mut w = match self.w.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        match event {
            ProcessedEvent::Line { ts, source, raw } => {
                let ts = fmt_ts(*ts);
                let src = fmt_source(source);
                let _ = writeln!(w, "[{ts}] {src} │ {raw}");
            }
            ProcessedEvent::System { ts, level, message } => {
                let ts = fmt_ts(*ts);
                let lvl = fmt_level(*level);
                let _ = writeln!(w, "[{ts}] [SYS] {lvl} ▸ {message}");
            }
        }
    }
}

fn fmt_ts(ts: SystemTime) -> String {
    let dt: DateTime<Utc> = ts.into();
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn fmt_level(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRC",
        LogLevel::Debug => "DBG",
        LogLevel::Info => "INF",
        LogLevel::Warn => "WRN",
        LogLevel::Error => "ERR",
    }
}

fn fmt_source(source: &SourceId) -> String {
    format!("[{}]", source.label())
}
