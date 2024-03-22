use dioxus::prelude::ScopeState;
use std::{future::Future, sync::Arc};
use tokio::sync::watch;

pub fn use_theramin_routine<T, F>(
    cx: &ScopeState,
    init_ch: T,
    routine: impl FnOnce(WatchTx<T>) -> F,
) -> &mut UseWatchRx<T>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
    T: Send + Sync + 'static,
{
    cx.use_hook(|| {
        let (tx, rx) = watch::channel(init_ch);
        tokio::spawn(routine(WatchTx {
            update: cx.schedule_update(),
            tx,
        }));
        UseWatchRx { rx }
    })
}

pub struct WatchTx<T> {
    update: Arc<dyn Fn() + Send + Sync + 'static>,
    tx: watch::Sender<T>,
}

pub struct UseWatchRx<T> {
    rx: watch::Receiver<T>,
}

impl<T> WatchTx<T> {
    pub fn send(&self, value: T) -> Result<(), watch::error::SendError<T>> {
        let res = self.tx.send(value);
        if res.is_ok() {
            (self.update)();
        }
        res
    }
}

impl<T> UseWatchRx<T> {
    pub fn read(&mut self) -> Result<watch::Ref<T>, watch::error::RecvError> {
        match self.rx.has_changed() {
            Ok(true) => Ok(self.rx.borrow_and_update()),
            Ok(false) => Ok(self.rx.borrow()),
            Err(e) => Err(e),
        }
    }
}
