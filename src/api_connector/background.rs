use std::sync::Arc;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;

use crate::api_connector::FetchStrategy;
use crate::api_connector::fetch_strategy::{TokenError, TokenSuccess};
use crate::api_connector::lock_around::FetchBehavior;
use crate::api_connector::lock_around::lock_around;

#[derive(Debug)]
pub struct BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    token_value: RwLock<Option<String>>,
    sender: mpsc::Sender<()>,
    config: T::Config,
    handle: Mutex<Option<JoinHandle<()>>>,
    lock_strategy: FetchBehavior,
}

impl<T> BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    pub fn new(lock_strategy: FetchBehavior, config: T::Config) -> Arc<Self> {
        // initialize the struct
        let (sender, receiver) = mpsc::channel();
        let token_value = RwLock::new(None);

        let fetcher = Arc::new(Self {
            sender,
            token_value,
            config,
            lock_strategy,
            handle: Mutex::new(None),
        });
        let self_clone = Arc::clone(&fetcher);

        // Spawn the inner thread
        let handle = thread::spawn(move || self_clone.background_job(receiver));
        *fetcher.handle.lock() = Some(handle);

        fetcher
    }

    pub fn exit(&self) {
        let _ = self.sender.send(());
    }

    pub fn background_job(&self, receiver: mpsc::Receiver<()>) {
        let mut context = T::init_context(&self.config);
        let mut wait_duration = Duration::ZERO; // First fetch happens immediately

        loop {
            // If we receive anything from a mpsc channel, it means we need to quit
            // the loop, otherwise we wait the timeout time
            let end_signal = receiver.recv_timeout(wait_duration);

            if !matches!(end_signal, Err(mpsc::RecvTimeoutError::Timeout)) {
                tracing::debug!("Token refresh background job shutting down");
                break;
            }

            let (mut guard, result) = lock_around(&self.token_value, self.lock_strategy, || {
                T::fetch(&self.config, &mut context)
            });

            wait_duration = match result {
                Ok(TokenSuccess { token, duration }) => {
                    *guard = Some(token);
                    duration
                }
                Err(TokenError {
                    error_message,
                    duration,
                }) => {
                    drop(guard);
                    tracing::warn!("Token refresh failed: {:?}", error_message);
                    duration
                }
            }
        }
    }

    pub fn acquire_read(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.token_value.read()
    }
}

impl<T> Drop for BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    fn drop(&mut self) {
        self.exit();
        if let Some(handle) = self.handle.lock().take() {
            let _ = handle.join();
        }
    }
}
