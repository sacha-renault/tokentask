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

#[derive(Debug, Default, bon::Builder)]
pub struct ConnectionConfiguration {}

struct DefaultHandler;

impl Handlers for DefaultHandler {
    type Config = ConnectionConfiguration;
    type Actions = Actions;
    type States = States;

    fn connect(config: &Self::Config) -> Self::States {
        println!("Connecting");
        Self::States::Connected("123456".to_string())
    }

    fn refresh(config: &Self::Config) -> Self::States {
        println!("Refreshing");
        Self::States::Connected("123456".to_string())
    }

    fn choose_action(previous_state: &Self::States, current_state: &Self::States) -> Self::Actions {
        match (previous_state, current_state) {
            (_, Self::States::Init) => Self::Actions::Connect,
            (_, Self::States::Connected(_)) => Self::Actions::Refresh,
            _ => todo!("choose_action"),
        }
    }

    fn execute(config: &Self::Config, action: Self::Actions) -> Self::States {
        match action {
            Self::Actions::Connect => Self::connect(config),
            Self::Actions::Refresh => Self::refresh(config),
        }
    }

    fn get_token_from_state(state: &Self::States) -> Option<&str> {
        match state {
            Self::States::Connected(token) => Some(&token),
            _ => None,
        }
    }
}

fn main() {
    let config = ConnectionConfiguration::builder().build();
    let a = ConnectionHandler::<DefaultHandler>::new(config);
    thread::sleep(Duration::from_secs_f32(1.5));
}
