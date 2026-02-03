use tokio::sync::watch;

#[derive(Clone)]
pub struct Shutdown {
    rx: watch::Receiver<bool>,
}

#[derive(Clone)]
pub struct ShutdownHandle {
    tx: watch::Sender<bool>,
}

pub fn shutdown_channel() -> (Shutdown, ShutdownHandle) {
    let (tx, rx) = watch::channel(false);
    (Shutdown { rx }, ShutdownHandle { tx })
}

impl Shutdown {
    pub fn is_triggered(&self) -> bool {
        *self.rx.borrow()
    }

    pub async fn changed(&mut self) -> bool {
        self.rx.changed().await.is_ok()
    }
}

impl ShutdownHandle {
    pub fn trigger(&self) {
        let _ = self.tx.send(true);
    }
}
