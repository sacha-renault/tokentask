mod api_connector;
mod oauth;
mod utils;

use std::{thread, time::Duration};

use api_connector::{ConnectionHandler, FetchStrategy};

#[derive(Debug, Clone, Default)]
pub enum States {
    #[default]
    Init,
    Connected(String),
    Disconnected,
}

#[derive(Debug, Copy, Clone)]
pub enum Actions {
    Connect,
    Refresh,
}

struct MyHandler;

impl FetchStrategy for MyHandler {
    type Config = ();
    type States = States;
    type Actions = Actions;

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

    fn get_wait_duration(state: &Self::States) -> Duration {
        Duration::from_secs(5)
    }

    fn choose_action(current: &Self::States) -> Self::Actions {
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
    let a = ConnectionHandler::<MyHandler>::new(());

    let token = a.get_token();
    println!("Token: {token:?}");
    thread::sleep(Duration::from_secs_f32(1.5));
    let token = a.get_token();
    println!("Token: {token:?}");
}
