use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub trait Handlers: Send + Sync + 'static {
    type Config: Send + Sync + 'static;
    type States: Default;
    type Actions: Copy;

    fn connect(config: &Self::Config) -> Self::States;
    fn refresh(config: &Self::Config) -> Self::States;
    fn execute(config: &Self::Config, action: Self::Actions) -> Self::States;
    fn choose_action(previous_state: &Self::States, current_state: &Self::States) -> Self::Actions;
    fn get_token_from_state(state: &Self::States) -> Option<&str>;
}

#[derive(Debug)]
pub struct BackgroundTokenFetch<T>
where
    T: Handlers,
{
    sender: mpsc::Sender<()>,
    token_changed: AtomicBool,
    token_value: Mutex<Option<String>>,
    config: T::Config,
    _p: PhantomData<T>,
}

impl<T> BackgroundTokenFetch<T>
where
    T: Handlers,
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
        let mut previous_state = T::States::default();
        let mut current_state = T::States::default();

        loop {
            if let Err(mpsc::RecvTimeoutError::Timeout) =
                receiver.recv_timeout(Duration::from_millis(100))
            {
                let action = T::choose_action(&previous_state, &current_state);
                previous_state = current_state;
                current_state = T::execute(&self.config, action);
                self.maybe_set_token(&current_state);
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

pub struct ConnectionHandler<T>
where
    T: Handlers,
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
    T: Handlers,
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
                return None;
            };
            let mut last_token_ref = self.last_token_value.borrow_mut();
            *last_token_ref = new_value;
        }
        self.last_token_value.borrow().clone()
    }
}

impl<T> Drop for ConnectionHandler<T>
where
    T: Handlers,
{
    fn drop(&mut self) {
        self.inner.exit();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
