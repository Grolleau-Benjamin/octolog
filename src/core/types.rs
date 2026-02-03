use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceId {
    pub port: String,
    pub alias: Option<String>,
}

impl SourceId {
    pub fn label(&self) -> String {
        match &self.alias {
            Some(s) => s.clone(),
            None => self.port.clone(),
        }
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    LogLine {
        source: SourceId,
        ts: SystemTime,
        raw: String,
    },
    System {
        level: LogLevel,
        message: String,
    },
}
