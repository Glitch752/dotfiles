use std::sync::Arc;

use tauri::async_runtime::JoinHandle;

/// A debouncer using async tasks.
pub struct Debouncer<T> {
    callback: Arc<dyn Fn(T) + Send + Sync>,
    delay: std::time::Duration,
    task: Option<JoinHandle<()>>,
    last_value: Option<T>,
}

impl<T> Debouncer<T> {
    pub fn new<F>(delay: std::time::Duration, callback: F) -> Self
        where F: Fn(T) + Send + Sync + 'static {
        Debouncer {
            callback: Arc::new(callback),
            delay,
            task: None,
            last_value: None,
        }
    }

    pub fn call(&mut self, value: T)
    where
        T: Send + 'static + Clone,
    {
        self.last_value = Some(value.clone());
        if let Some(task) = self.task.take() {
            task.abort();
        }
        let delay = self.delay;
        let callback = self.callback.clone();
        let last_value = self.last_value.clone();
        self.task = Some(tauri::async_runtime::spawn(async move {
            tokio::time::sleep(delay).await;
            if let Some(val) = last_value {
                (callback)(val);
            }
        }));
    }
}