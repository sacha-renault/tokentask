use std::marker::PhantomData;
use std::sync::{Arc, RwLockWriteGuard};
use std::sync::{RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::api_connector::FetchStrategy;
use crate::api_connector::fetch_strategy::{TokenError, TokenSuccess};

#[derive(Debug)]
pub struct BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    pub(super) token_value: RwLock<Option<String>>,
    sender: mpsc::Sender<()>,
    config: T::Config,
    _p: PhantomData<T>,
}

impl<T> BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    pub fn new_job(config: T::Config) -> (Arc<Self>, JoinHandle<()>) {
        // initialize the struct
        let (sender, receiver) = mpsc::channel();
        let token_value = RwLock::new(None);

        let fetcher = Arc::new(Self {
            sender,
            token_value,
            config,
            _p: PhantomData,
        });
        let self_clone = Arc::clone(&fetcher);

        // Spawn the inner thread
        let handle = thread::spawn(move || self_clone.background_job(receiver));
        (fetcher, handle)
    }

    pub fn exit(&self) {
        let _ = self.sender.send(());
    }

    pub fn background_job(&self, receiver: mpsc::Receiver<()>) {
        let mut context = T::init_context(&self.config).unwrap(); // TODO ??!!
        let mut wait_duration = Duration::ZERO; // First fetch happens immediately

        loop {
            // If we receive anything from a mpsc channel, it means we need to quit
            // the loop, otherwise we wait the timeout time
            let end_signal = receiver.recv_timeout(wait_duration);

            if !matches!(end_signal, Err(mpsc::RecvTimeoutError::Timeout)) {
                break;
            }

            // We need to acquire the write lock BEFORE fetching the new token.
            // This prevents other threads from using the old token while we're
            // in the process of refreshing it (which would invalidate the old token
            // and cause 401 errors).
            let mut guard = self.acquire_lock();

            wait_duration = match T::fetch(&self.config, &mut context) {
                Ok(TokenSuccess { token, duration }) => {
                    *guard = Some(token);
                    duration
                }
                Err(TokenError {
                    error_message,
                    duration,
                }) => {
                    tracing::warn!("Token refresh failed: {:?}", error_message);
                    duration
                }
            }
        }
    }

    fn acquire_lock(&self) -> RwLockWriteGuard<'_, Option<String>> {
        self.token_value
            .write()
            .expect("Token lock poisoned - cannot refresh token")
    }
}
