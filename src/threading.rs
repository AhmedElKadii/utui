use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct AsyncTask<T> {
    rx: Receiver<T>,
}

impl<T: Send + 'static> AsyncTask<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let result = f();
            let _ = tx.send(result); // If main thread dropped it, ignore error
        });

        Self { rx }
    }

    pub fn poll(&self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}
