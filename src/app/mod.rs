use crate::cli::CliArgs;
use crate::config::Config;
use crate::core::{AppError, AppResult};
use crate::processing::LogProcessor;
use crate::runtime::engine::LineFilter;
use crate::runtime::{Engine, shutdown_channel};
use crate::sinks::{FileSink, StdoutSink, spawn_fanout, spawn_sink_worker};
use crate::sources::serial;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run(args: CliArgs) -> AppResult<()> {
    let cfg = Config::try_from(args)?;

    if cfg.list {
        let ports = serial::scan::list_available_ports(None)?;
        if ports.is_empty() {
            println!("No serial ports found.");
            return Ok(());
        }
        for p in ports {
            println!("{p}");
        }
        return Ok(());
    }

    let (shutdown, shutdown_handle) = shutdown_channel();

    let mut sink_txs = Vec::new();
    let mut sink_handles = Vec::new();

    let stdout_sink = Arc::new(StdoutSink::new().with_highlights(cfg.highlight.clone()));
    let (stdout_tx, stdout_h) = spawn_sink_worker(stdout_sink, cfg.runtime.event_bus_capacity);
    sink_txs.push(stdout_tx);
    sink_handles.push(stdout_h);

    if let Some(path) = cfg.output {
        let file_sink = Arc::new(FileSink::new(path).map_err(|e| AppError::Config(e.to_string()))?);
        let (file_tx, file_h) = spawn_sink_worker(file_sink, cfg.runtime.event_bus_capacity);
        sink_txs.push(file_tx);
        sink_handles.push(file_h);
    }

    let (processed_tx, processed_rx) = mpsc::channel(cfg.runtime.event_bus_capacity);

    let fanout_task = spawn_fanout(processed_rx, sink_txs);

    let processor = LogProcessor::new();
    let filter = LineFilter::new(cfg.filter.clone(), cfg.exclude.clone());
    let engine = Engine::new(processor, processed_tx, shutdown.clone()).with_filter(filter);

    let (tx, rx) = mpsc::channel(cfg.runtime.event_bus_capacity);
    let engine_task = tokio::spawn(engine.run(rx));

    let source_tasks = serial::SerialSource::new(cfg.ports, tx, shutdown.clone()).spawn();

    tokio::signal::ctrl_c()
        .await
        .map_err(|e| AppError::Runtime(e.to_string()))?;

    shutdown_handle.trigger();

    for t in source_tasks {
        let _ = t.await;
    }

    engine_task
        .await
        .map_err(|e| AppError::Runtime(e.to_string()))??;

    let _ = fanout_task.await;

    for h in sink_handles {
        let _ = h.await;
    }

    Ok(())
}
