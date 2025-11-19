use std::sync::Arc;

use crate::api_connector::{FetchStrategy, background::BackgroundTokenFetch};

pub struct ConnectionHandler<T>
where
    T: FetchStrategy,
{
    inner: Arc<BackgroundTokenFetch<T>>,
}

impl<T> ConnectionHandler<T>
where
    T: FetchStrategy,
{
    pub fn new(config: T::Config) -> Self {
        let inner = BackgroundTokenFetch::new_job(config);

        Self { inner }
    }

    /// Execute a closure with a valid token, preventing refresh during use
    pub fn with_token<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&str) -> R,
    {
        let token_guard = self.inner.acquire_read();
        token_guard.as_ref().map(|token| f(token))
    }
}
