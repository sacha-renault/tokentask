mod token_fetch;

use std::{thread, time::Duration};

use token_fetch::ConnectionHandler;

use crate::token_fetch::Handlers;

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
    let a = ConnectionHandler::<MyHandler>::new(());

    let token = a.get_token();
    thread::sleep(Duration::from_secs_f32(1.5));
}
