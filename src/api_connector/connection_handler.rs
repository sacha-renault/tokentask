use std::sync::Arc;
use std::thread::JoinHandle;

use crate::api_connector::{FetchStrategy, background::BackgroundTokenFetch};

pub struct ConnectionHandler<T>
where
    T: FetchStrategy,
{
    inner: Arc<BackgroundTokenFetch<T>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl<T> ConnectionHandler<T>
where
    T: FetchStrategy,
{
    pub fn new(config: T::Config) -> Self {
        let (inner, thread_handle) = BackgroundTokenFetch::new_job(config);

        Self {
            inner,
            thread_handle: Some(thread_handle),
        }
    }

    /// Execute a closure with a valid token, preventing refresh during use
    pub fn with_token<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&str) -> R,
    {
        let token_guard = self.inner.token_value.read().ok()?;
        token_guard.as_ref().map(|token| f(token))
    }
}

impl<T> Drop for ConnectionHandler<T>
where
    T: FetchStrategy,
{
    fn drop(&mut self) {
        self.inner.exit();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
