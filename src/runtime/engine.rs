use crate::core::{AppEvent, AppResult, LogLevel};
use crate::processing::{LogProcessor, ProcessedEvent};
use crate::runtime::Shutdown;
use std::time::SystemTime;
use tokio::sync::mpsc;

#[derive(Clone, Default)]
pub struct LineFilter {
    include: Option<String>,
    exclude: Vec<String>,
}

impl LineFilter {
    pub fn new(include: Option<String>, exclude: Vec<String>) -> Self {
        let include = include.and_then(|s| {
            let t = s.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        });

        let exclude = exclude
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        Self { include, exclude }
    }

    pub fn allows(&self, raw: &str) -> bool {
        if let Some(inc) = &self.include {
            if !raw.contains(inc) {
                return false;
            }
        }
        for ex in &self.exclude {
            if raw.contains(ex) {
                return false;
            }
        }
        true
    }
}

pub struct Engine {
    processor: LogProcessor,
    out: mpsc::Sender<ProcessedEvent>,
    shutdown: Shutdown,
    dropped: u64,
    filter: LineFilter,
}

impl Engine {
    pub fn new(
        processor: LogProcessor,
        out: mpsc::Sender<ProcessedEvent>,
        shutdown: Shutdown,
    ) -> Self {
        Self {
            processor,
            out,
            shutdown,
            dropped: 0,
            filter: LineFilter::default(),
        }
    }

    pub fn with_filter(mut self, filter: LineFilter) -> Self {
        self.filter = filter;
        self
    }

    pub async fn run(mut self, mut rx: mpsc::Receiver<AppEvent>) -> AppResult<()> {
        loop {
            if self.shutdown.is_triggered() {
                break;
            }

            tokio::select! {
                _ = self.shutdown.changed() => {
                    if self.shutdown.is_triggered() {
                        break;
                    }
                }
                evt = rx.recv() => {
                    let Some(evt) = evt else { break; };

                    if let AppEvent::LogLine { raw, .. } = &evt {
                        if !self.filter.allows(raw) {
                            continue;
                        }
                    }

                    let out = self.processor.process(evt)?;
                    self.publish(out);
                }
            }
        }

        Ok(())
    }

    fn publish(&mut self, event: ProcessedEvent) {
        if self.out.is_closed() {
            self.dropped = 0;
            return;
        }

        if self.dropped > 0 {
            let warn = ProcessedEvent::System {
                ts: SystemTime::now(),
                level: LogLevel::Warn,
                message: format!(
                    "dropped {} processed events (sink backpressure)",
                    self.dropped
                ),
            };

            match self.out.try_send(warn) {
                Ok(_) => self.dropped = 0,
                Err(mpsc::error::TrySendError::Full(_)) => {}
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    self.dropped = 0;
                    return;
                }
            }
        }

        match self.out.try_send(event) {
            Ok(_) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped = self.dropped.saturating_add(1);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.dropped = 0;
            }
        }
    }
}
