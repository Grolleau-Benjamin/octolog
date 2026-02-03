pub mod file;
pub mod stdout;

use crate::processing::ProcessedEvent;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub trait EventSink: Send + Sync + 'static {
    fn emit(&self, event: &ProcessedEvent);
}

pub fn spawn_sink_worker(
    sink: Arc<dyn EventSink>,
    capacity: usize,
) -> (mpsc::Sender<Arc<ProcessedEvent>>, JoinHandle<()>) {
    let (tx, mut rx) = mpsc::channel::<Arc<ProcessedEvent>>(capacity);

    let handle = tokio::task::spawn_blocking(move || {
        while let Some(evt) = rx.blocking_recv() {
            sink.emit(&evt);
        }
    });

    (tx, handle)
}

pub fn spawn_fanout(
    mut rx: mpsc::Receiver<ProcessedEvent>,
    sinks: Vec<mpsc::Sender<Arc<ProcessedEvent>>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(evt) = rx.recv().await {
            let evt = Arc::new(evt);

            for tx in sinks.iter() {
                let _ = tx.try_send(evt.clone());
            }
        }
    })
}

pub use file::FileSink;
pub use stdout::StdoutSink;
