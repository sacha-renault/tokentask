use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::api_connector::FetchStrategy;

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
        let mut state = T::States::default();
        let mut wait_duration;

        loop {
            wait_duration = T::get_wait_duration(&state);
            if let Err(mpsc::RecvTimeoutError::Timeout) = receiver.recv_timeout(wait_duration) {
                let action = T::choose_action(&state);
                state = T::execute(&self.config, action);
                self.maybe_set_token(&state);
            } else {
                break;
            }
        }
    }

    pub fn maybe_set_token(&self, state: &T::States) {
        if let Some(token) = T::get_token_from_state(state) {
            if let Ok(mut guard) = self.token_value.lock() {
                *guard = Some(token.to_string());
                self.token_changed.store(true, Ordering::Release);
            }
        }
    }
}
