# tokenfetch

Background token refresh without the boilerplate.

## What is this?

You know when you need to keep refreshing OAuth tokens, API keys, or database connections in the background? And you end up writing the same threading + state machine code every time?

This handles that. You define the states and actions, it runs the loop.

## Example

```rust
use tokenfetch::{Handlers, ConnectionHandler};

#[derive(Clone, Default)]
enum States {
    #[default]
    Init,
    Connected(String),
    Disconnected,
}

#[derive(Copy, Clone)]
enum Actions {
    Connect,
    Refresh,
}

struct MyHandler;

impl Handlers for MyHandler {
    type Config = ();
    type States = States;
    type Actions = Actions;

    fn connect(config: &Self::Config) -> Self::States {
        unreachable!()
    }

    fn refresh(config: &Self::Config) -> Self::States {
        unreachable!()
    }

    fn execute(config: &Self::Config, action: Self::Actions) -> Self::States {
        match action {
            Actions::Connect => {
                // Do your OAuth/API call here
                States::Connected("token_123".into())
            }
            Actions::Refresh => {
                // Refresh logic
                States::Connected("new_token".into())
            }
        }
    }

    fn choose_action(prev: &Self::States, current: &Self::States) -> Self::Actions {
        match current {
            States::Init => Actions::Connect,
            States::Connected(_) => Actions::Refresh,
            States::Disconnected => Actions::Connect,
        }
    }

    fn get_token_from_state(state: &Self::States) -> Option<&str> {
        match state {
            States::Connected(token) => Some(token),
            _ => None,
        }
    }
}

fn main() {
    let handler = ConnectionHandler::<MyHandler>::new(());
    
    // Token refreshes in background
    loop {
        if let Some(token) = handler.get_token() {
            println!("Using: {}", token);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

## TODO

- [ ] Add exponential backoff support
- [ ] Configurable polling interval
- [ ] Metrics/callbacks for state transitions
- [ ] Better error handling
- [ ] Examples for common cases (OAuth, JWT, etc.)