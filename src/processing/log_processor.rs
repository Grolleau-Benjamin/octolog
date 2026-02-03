use crate::core::{AppEvent, AppResult, LogLevel, SourceId};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessedEvent {
    Line {
        ts: SystemTime,
        source: SourceId,
        raw: String,
    },
    System {
        ts: SystemTime,
        level: LogLevel,
        message: String,
    },
}

#[derive(Clone, Default)]
pub struct LogProcessor;

impl LogProcessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process(&self, event: AppEvent) -> AppResult<ProcessedEvent> {
        Ok(match event {
            AppEvent::LogLine { source, ts, raw } => ProcessedEvent::Line { ts, source, raw },
            AppEvent::System { level, message } => ProcessedEvent::System {
                ts: SystemTime::now(),
                level,
                message,
            },
        })
    }
}
