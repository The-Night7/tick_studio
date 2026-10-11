use std::sync::Arc;

use parking_lot::Mutex;
use tokio::task::JoinHandle;


#[derive(Debug, Clone)]
pub struct CancelToken {
    tx: tokio::sync::watch::Sender<bool>,
    _keepalive: tokio::sync::watch::Receiver<bool>,
}

impl CancelToken {
    fn new() -> Self {
        let (tx, rx) = tokio::sync::watch::channel(false);
        Self {
            tx,
            _keepalive: rx,
        }
    }

    pub fn cancel(&self) {
        let _ = self.tx.send(true);
    }

    pub fn is_cancelled(&self) -> bool {
        *self.tx.borrow()
    }

    pub async fn cancelled(&self) {
        let mut rx = self.tx.subscribe();
        let _ = rx.wait_for(|cancelled| *cancelled).await;
    }
}

pub struct Task<S> {
    state: Arc<Mutex<S>>,
    cancel: CancelToken,
    handle: Option<JoinHandle<()>>,
}

impl<S: Send + 'static> Task<S> {
    pub fn spawn<F, Fut>(initial: S, f: F) -> Self
    where
        F: FnOnce(Arc<Mutex<S>>, CancelToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let state = Arc::new(Mutex::new(initial));
        let cancel = CancelToken::new();
        let worker_state = Arc::clone(&state);
        let worker_cancel = cancel.clone();
        let handle = tokio::spawn(async move {
            f(worker_state, worker_cancel).await;
        });
        Self {
            state,
            cancel,
            handle: Some(handle),
        }
    }

    pub fn state(&self) -> &Arc<Mutex<S>> {
        &self.state
    }

    pub fn cancel_token(&self) -> &CancelToken {
        &self.cancel
    }

    pub fn is_running(&self) -> bool {
        self.handle.as_ref().is_some_and(|h| !h.is_finished())
    }

    pub fn stop(&self) {
        self.cancel.cancel();
    }

    pub fn abort(&self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }
}

impl<S> Drop for Task<S> {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}