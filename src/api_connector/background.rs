use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;

use crate::api_connector::FetchStrategy;
use crate::api_connector::fetch_strategy::{TokenError, TokenSuccess};
use crate::api_connector::lock_around::LockBehavior;
use crate::api_connector::lock_around::lock_around;

pub enum ThreadMessage {
    Stop,
    InvalidToken,
}

#[derive(Debug)]
pub struct BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    token_value: RwLock<Option<String>>,
    sender: mpsc::Sender<ThreadMessage>,
    config: T::Config,
    handle: Mutex<Option<JoinHandle<()>>>,
    lock_behavior: LockBehavior,
}

impl<T> BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    /// Starts the background token fetch service.
    ///
    /// **Note**: This performs an initial synchronous token fetch before
    /// spawning the background thread, so it may block briefly.
    pub fn init(lock_behavior: LockBehavior, config: T::Config) -> Arc<Self> {
        // initialize the struct
        let (sender, receiver) = mpsc::channel();
        let token_value = RwLock::new(None);

        let fetcher = Arc::new(Self {
            sender,
            token_value,
            config,
            lock_behavior,
            handle: Mutex::new(None),
        });
        let self_clone = Arc::clone(&fetcher);

        // Spawn the inner thread
        let mut context = T::init_context(&fetcher.config);
        match fetcher.background_job(&receiver, Duration::ZERO, &mut context) {
            ControlFlow::Continue(duration) => {
                let handle =
                    thread::spawn(move || self_clone.background_loop(receiver, duration, context));
                *fetcher.handle.lock() = Some(handle);
            }
            ControlFlow::Break(_) => tracing::warn!(
                "Thread didn't start, mpsc channels disconnected after initial fetch"
            ),
        }

        fetcher
    }

    pub fn exit(&self) {
        let _ = self.sender.send(ThreadMessage::Stop);
    }

    pub fn invalid_token(&self) {
        let _ = self.sender.send(ThreadMessage::InvalidToken);
    }

    pub fn background_loop(
        &self,
        receiver: mpsc::Receiver<ThreadMessage>,
        init_duration: Duration,
        mut context: T::Context,
    ) {
        tracing::info!("Starting background fetch token job");
        let mut wait_duration = init_duration;

        loop {
            tracing::debug!("Waiting : {:?} before querying new token", wait_duration);
            match self.background_job(&receiver, wait_duration, &mut context) {
                ControlFlow::Continue(duration) => {
                    wait_duration = duration;
                }
                ControlFlow::Break(_) => break,
            }
        }
    }

    pub fn background_job(
        &self,
        receiver: &mpsc::Receiver<ThreadMessage>,
        wait_duration: Duration,
        context: &mut T::Context,
    ) -> ControlFlow<(), Duration> {
        tracing::debug!("About to query new token");

        // If we receive anything from a mpsc channel, it means we need to quit
        // the loop, otherwise we wait the timeout time
        let end_signal = receiver.recv_timeout(wait_duration);

        let lock_strategy = match end_signal {
            // We receive End signal or we disconnected => we need to stop
            Ok(ThreadMessage::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                return ControlFlow::Break(());
            }

            // Continue normally
            Err(mpsc::RecvTimeoutError::Timeout) => self.lock_behavior,

            // Token is known invalid - block all requests until we get a new one
            // Ok(ThreadMessage::InvalidToken) => LockBehavior::HoldDuringOperation,
            // Rn commented because client could call this and this would break
            // Exponential wait ...
            Ok(ThreadMessage::InvalidToken) => self.lock_behavior,
        };

        let (mut guard, result) = lock_around(&self.token_value, lock_strategy, || {
            T::fetch(&self.config, context)
        });

        let wait_duration = match result {
            Ok(TokenSuccess {
                token,
                fetch_after: duration,
            }) => {
                tracing::debug!("Token acquired.");
                *guard = Some(token);
                duration
            }
            Err(TokenError {
                error_message,
                retry_after: duration,
            }) => {
                drop(guard);
                tracing::warn!("Token refresh failed: {:?}", error_message);
                duration
            }
        };

        return ControlFlow::Continue(wait_duration);
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
