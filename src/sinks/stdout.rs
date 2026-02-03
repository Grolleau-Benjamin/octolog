use crate::core::{LogLevel, SourceId};
use crate::processing::ProcessedEvent;
use crate::sinks::EventSink;

use chrono::{DateTime, SecondsFormat, Utc};
use owo_colors::OwoColorize;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;

#[derive(Clone, Default)]
pub struct StdoutSink {
    highlights: Vec<(String, String)>, // (pattern, colored)
}

impl StdoutSink {
    pub fn new() -> Self {
        Self {
            highlights: Vec::new(),
        }
    }

    pub fn with_highlights(mut self, highlights: Vec<String>) -> Self {
        self.highlights = highlights
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|p| {
                let colored = format!("{}", p.yellow().bold());
                (p, colored)
            })
            .collect();
        self
    }
}

impl EventSink for StdoutSink {
    fn emit(&self, event: &ProcessedEvent) {
        match event {
            ProcessedEvent::Line { ts, source, raw } => {
                let ts = fmt_ts(*ts).dimmed().to_string();
                let src = fmt_source(source);
                let raw = apply_highlights(raw, &self.highlights);
                println!("[{ts}] {src} │ {raw}");
            }
            ProcessedEvent::System { ts, level, message } => {
                let ts = fmt_ts(*ts).dimmed().to_string();
                let sys = "[SYS]".magenta().bold().to_string();
                let lvl = fmt_level(*level);
                eprintln!("[{ts}] {sys} {lvl} ▸ {message}");
            }
        }
    }
}

fn fmt_ts(ts: SystemTime) -> String {
    let dt: DateTime<Utc> = ts.into();
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn fmt_level(level: LogLevel) -> String {
    match level {
        LogLevel::Trace => "TRC".bright_black().to_string(),
        LogLevel::Debug => "DBG".cyan().to_string(),
        LogLevel::Info => "INF".green().to_string(),
        LogLevel::Warn => "WRN".yellow().to_string(),
        LogLevel::Error => "ERR".red().to_string(),
    }
}

fn fmt_source(source: &SourceId) -> String {
    let label = source.label();
    let tag = format!("[{label}]");

    let key = match &source.alias {
        Some(a) => format!("{}|{a}", source.port),
        None => format!("{}|", source.port),
    };

    let (r, g, b) = color_from_key(&key);
    tag.truecolor(r, g, b).bold().to_string()
}

fn color_from_key(key: &str) -> (u8, u8, u8) {
    let mut h = DefaultHasher::new();
    key.hash(&mut h);
    let x = h.finish();

    let mut r = (x & 0xFF) as u8;
    let mut g = ((x >> 8) & 0xFF) as u8;
    let mut b = ((x >> 16) & 0xFF) as u8;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max.wrapping_sub(min) < 48 {
        r = r.wrapping_add(80);
        g = g.wrapping_add(160);
        b = b.wrapping_add(40);
    }

    r = 64 + (r % 161);
    g = 64 + (g % 161);
    b = 64 + (b % 161);

    (r, g, b)
}

fn apply_highlights(raw: &str, patterns: &[(String, String)]) -> String {
    if patterns.is_empty() {
        return raw.to_string();
    }

    let mut s = raw.to_string();
    for (p, colored) in patterns {
        if p.is_empty() {
            continue;
        }
        if s.contains(p) {
            s = s.replace(p, colored);
        }
    }
    s
}
