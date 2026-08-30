use std::sync::mpsc::{self, Receiver};
use std::thread;
use crate::App;
use crate::error::AppError;

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

pub fn poll_task<T: Send + 'static>(
    app: &mut App,
    task_field: impl Fn(&mut App) -> &mut Option<AsyncTask<Result<T, AppError>>>,
    on_pending: impl FnOnce(&mut App),
    on_result: impl FnOnce(&mut App, Result<T, AppError>),
) {
    let Some(task) = task_field(app).as_ref() else {
        return;
    };
    let Some(result) = task.poll() else {
        on_pending(app);
        return;
    };
    *task_field(app) = None;
    on_result(app, result);
}
