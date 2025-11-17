use std::cell::RefCell;
use std::sync::{Arc, atomic::Ordering};
use std::thread::JoinHandle;

use crate::api_connector::{FetchStrategy, background::BackgroundTokenFetch};

pub struct ConnectionHandler<T>
where
    T: FetchStrategy,
{
    inner: Arc<BackgroundTokenFetch<T>>,
    thread_handle: Option<JoinHandle<()>>,

    // More convinient to have RefCell to avoid forcing
    // Mutability everywhere. Be carefull accessing this
    // Only in get_token or there might be problems ...
    // ```
    // let ref = self.last_token_value.borrow_mut();
    // let token = self.get_token() // this panics
    //```
    last_token_value: RefCell<Option<String>>,
}

impl<T> ConnectionHandler<T>
where
    T: FetchStrategy,
{
    pub fn new(config: T::Config) -> Self {
        let (inner, thread_handle) = BackgroundTokenFetch::new_job(config);
        let last_token_value = RefCell::new(None);

        Self {
            inner,
            thread_handle: Some(thread_handle),
            last_token_value,
        }
    }

    pub fn get_token(&self) -> Option<String> {
        if self.inner.token_changed.load(Ordering::Acquire) {
            let new_value = if let Ok(lock) = self.inner.token_value.try_lock() {
                let value = lock.as_ref().cloned();
                self.inner.token_changed.store(false, Ordering::Relaxed);
                value
            } else {
                // Better none than panic tho ...
                return self.last_token_value.borrow().clone();
            };
            let mut last_token_ref = self.last_token_value.borrow_mut();
            *last_token_ref = new_value;
        }
        self.last_token_value.borrow().clone()
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
