//! Work that waits for a Model, done beside the window so that the
//! window stays alive meanwhile.

use std::sync::mpsc::{Receiver, channel};

/// One piece of work under way, and later its result.
pub struct Job<T> {
    result: Receiver<iibrain_core::Result<T>>,
}

impl<T: Send + 'static> Job<T> {
    /// Starts `work`: beside the window, or with `beside` off at once,
    /// as a test wants it.
    pub fn start(
        beside: bool,
        work: impl FnOnce() -> iibrain_core::Result<T> + Send + 'static,
    ) -> Self {
        let (done, result) = channel();
        if beside {
            std::thread::spawn(move || {
                let _ = done.send(work());
            });
        } else {
            let _ = done.send(work());
        }
        Self { result }
    }

    /// The result, once, when the work is done.
    pub fn take(&self) -> Option<iibrain_core::Result<T>> {
        self.result.try_recv().ok()
    }
}
