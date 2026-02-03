use crate::core::{AppEvent, LogLevel, ResolvedPortSpec, SourceId};
use crate::runtime::Shutdown;
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

#[cfg(target_os = "macos")]
use nix::sys::termios::{SetArg, cfmakeraw, tcgetattr, tcsetattr};
#[cfg(target_os = "macos")]
use std::os::fd::{AsRawFd, BorrowedFd};

const MAX_ACC_BYTES: usize = 64 * 1024;

pub struct SerialSource {
    ports: Vec<ResolvedPortSpec>,
    tx: mpsc::Sender<AppEvent>,
    shutdown: Shutdown,
    reconnect_delay: Duration,
}

impl SerialSource {
    pub fn new(
        ports: Vec<ResolvedPortSpec>,
        tx: mpsc::Sender<AppEvent>,
        shutdown: Shutdown,
    ) -> Self {
        Self {
            ports,
            tx,
            shutdown,
            reconnect_delay: Duration::from_secs(1),
        }
    }

    pub fn spawn(self) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::with_capacity(self.ports.len());

        for spec in self.ports {
            let tx = self.tx.clone();
            let shutdown = self.shutdown.clone();
            let reconnect_delay = self.reconnect_delay;

            handles.push(tokio::spawn(async move {
                run_port_loop(spec, tx, shutdown, reconnect_delay).await;
            }));
        }

        handles
    }
}

async fn run_port_loop(
    spec: ResolvedPortSpec,
    tx: mpsc::Sender<AppEvent>,
    mut shutdown: Shutdown,
    reconnect_delay: Duration,
) {
    let source = SourceId {
        port: spec.path.clone(),
        alias: spec.alias.clone(),
    };

    loop {
        if shutdown.is_triggered() {
            break;
        }

        let mut port = match open_serial(&spec).await {
            Ok(p) => p,
            Err(e) => {
                let _ = tx
                    .send(AppEvent::System {
                        level: LogLevel::Warn,
                        message: format!(
                            "serial open failed ({} @ {}): {}",
                            spec.path, spec.baud, e
                        ),
                    })
                    .await;
                sleep(reconnect_delay).await;
                continue;
            }
        };

        let _ = tx
            .send(AppEvent::System {
                level: LogLevel::Info,
                message: format!("connected: {} @ {}", spec.path, spec.baud),
            })
            .await;

        let disconnected = read_lines(&mut port, &source, &tx, &mut shutdown).await;

        if shutdown.is_triggered() {
            break;
        }

        if disconnected {
            sleep(reconnect_delay).await;
        }
    }
}

async fn open_serial(spec: &ResolvedPortSpec) -> Result<SerialStream, tokio_serial::Error> {
    let mut port = tokio_serial::new(&spec.path, spec.baud)
        .timeout(Duration::from_millis(100))
        .dtr_on_open(true)
        .open_native_async()?;

    #[cfg(target_os = "macos")]
    {
        let raw_fd = port.as_raw_fd();
        let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };

        if let Ok(mut t) = tcgetattr(fd) {
            cfmakeraw(&mut t);
            let _ = tcsetattr(fd, SetArg::TCSANOW, &t);
        }
    }

    let _ = port.write_data_terminal_ready(true);
    let _ = port.write_request_to_send(true);

    Ok(port)
}

async fn read_lines(
    port: &mut SerialStream,
    source: &SourceId,
    tx: &mpsc::Sender<AppEvent>,
    shutdown: &mut Shutdown,
) -> bool {
    let mut buf = [0u8; 2048];
    let mut acc: Vec<u8> = Vec::with_capacity(4096);

    loop {
        if shutdown.is_triggered() {
            return false;
        }

        tokio::select! {
            _ = shutdown.changed() => {
                if shutdown.is_triggered() {
                    return false;
                }
            }
            res = port.read(&mut buf) => {
                let n = match res {
                    Ok(0) => {
                        let _ = tx.send(AppEvent::System {
                            level: LogLevel::Warn,
                            message: "serial EOF".to_string(),
                        }).await;
                        return true;
                    }
                    Ok(n) => n,
                    Err(e) if is_transient_read_error(&e) => {
                        continue;
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::System {
                            level: LogLevel::Error,
                            message: format!("serial read failed: {e}"),
                        }).await;
                        return true;
                    }
                };

                acc.extend_from_slice(&buf[..n]);

                if acc.len() > MAX_ACC_BYTES {
                    acc.clear();
                    let _ = tx.send(AppEvent::System {
                        level: LogLevel::Warn,
                        message: format!(
                            "serial buffer overflow on {} (>{} bytes): dropping partial line",
                            source.label(),
                            MAX_ACC_BYTES
                        ),
                    }).await;
                }

                while let Some(raw) = try_pop_line(&mut acc) {
                    if raw.is_empty() {
                        continue;
                    }

                    if tx.send(AppEvent::LogLine {
                        source: source.clone(),
                        ts: SystemTime::now(),
                        raw,
                    }).await.is_err() {
                        return false;
                    }
                }
            }
        }
    }
}

fn is_transient_read_error(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
    )
}

fn try_pop_line(acc: &mut Vec<u8>) -> Option<String> {
    let pos = acc.iter().position(|&b| b == b'\n' || b == b'\r')?;

    let raw = String::from_utf8_lossy(&acc[..pos]).trim().to_string();

    acc.drain(..=pos);

    let mut k = 0usize;
    while k < acc.len() && (acc[k] == b'\n' || acc[k] == b'\r') {
        k += 1;
    }
    if k > 0 {
        acc.drain(..k);
    }

    Some(raw)
}
