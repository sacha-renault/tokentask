use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::api_connector::FetchStrategy;
use crate::api_connector::fetch_strategy::{RetryDuration, TokenSuccess};

#[derive(Debug)]
pub struct BackgroundTokenFetch<T>
where
    T: FetchStrategy,
{
    pub(super) token_changed: AtomicBool,
    pub(super) token_value: Mutex<Option<String>>,
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
        let token_changed = AtomicBool::new(false);
        let token_value = Mutex::new(None);

        let fetcher = Arc::new(Self {
            sender,
            token_changed,
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
        let mut context = T::init_context(&self.config);
        let mut wait_duration = Duration::ZERO; // First fetch happens immediately

        loop {
            // If we receive anything from a mpsc channel, it means we need to quit
            // the loop, otherwise we wait the timeout time
            let end_signal = receiver.recv_timeout(wait_duration);

            if let Err(mpsc::RecvTimeoutError::Timeout) = end_signal {
                match T::fetch(&self.config, &mut context) {
                    Ok(TokenSuccess { token, duration }) => {
                        self.set_token(token);
                        wait_duration = duration;
                    }
                    Err(RetryDuration(duration)) => wait_duration = duration,
                }
            } else {
                break;
            }
        }
    }

    pub fn set_token(&self, token: String) {
        if let Ok(mut guard) = self.token_value.lock() {
            *guard = Some(token.to_string());
            self.token_changed.store(true, Ordering::Release);
        }
    }
}
