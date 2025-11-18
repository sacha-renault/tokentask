use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api_connector::{ConnectionHandler, FetchStrategy};

#[derive(Debug, Clone, bon::Builder)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
}

#[derive(Debug, Clone)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub enum OAuthStates {
    #[default]
    Init,
    Connected {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: u64,
    },
    Disconnected,
    Error,
}

#[derive(Debug, Clone)]
pub enum OAuthActions {
    Connect,
    Refresh { refresh_token: String },
    Reconnect,
    HandleError,
}

pub struct OAuthStrategy;

impl OAuthStrategy {
    fn request_initial_token(config: &OAuthConfig) -> Result<TokenResponse, String> {
        todo!()
    }

    fn request_refresh_token(
        config: &OAuthConfig,
        refresh_token: &str,
    ) -> Result<TokenResponse, String> {
        todo!()
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn connect(config: &OAuthConfig) -> OAuthStates {
        let response = match Self::request_initial_token(config) {
            Ok(r) => r,
            Err(_) => {
                return OAuthStates::Error;
            }
        };

        OAuthStates::Connected {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_at: Self::now() + response.expires_in,
        }
    }

    fn refresh(config: &OAuthConfig, refresh_token: &str) -> OAuthStates {
        let response = match Self::request_refresh_token(config, refresh_token) {
            Ok(r) => r,
            Err(_) => {
                return OAuthStates::Disconnected;
            }
        };

        OAuthStates::Connected {
            access_token: response.access_token,
            refresh_token: response
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())),
            expires_at: Self::now() + response.expires_in,
        }
    }

    fn reconnect(config: &OAuthConfig) -> OAuthStates {
        std::thread::sleep(Duration::from_secs(5));

        let response = match Self::request_initial_token(config) {
            Ok(r) => r,
            Err(_) => {
                return OAuthStates::Disconnected;
            }
        };

        OAuthStates::Connected {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_at: Self::now() + response.expires_in,
        }
    }
}

impl FetchStrategy for OAuthStrategy {
    type Config = OAuthConfig;
    type States = OAuthStates;
    type Actions = OAuthActions;
    type Context = ();

    fn execute(
        config: &Self::Config,
        action: Self::Actions,
        context: &mut Self::Context,
    ) -> Self::States {
        match action {
            OAuthActions::Connect => Self::connect(config),
            OAuthActions::Refresh { refresh_token } => Self::refresh(config, &refresh_token),
            OAuthActions::Reconnect => Self::reconnect(config),
            OAuthActions::HandleError => OAuthStates::Disconnected,
        }
    }

    fn choose_action(state: &Self::States, context: &mut Self::Context) -> Self::Actions {
        match state {
            OAuthStates::Init => OAuthActions::Connect,
            OAuthStates::Connected {
                expires_at,
                refresh_token,
                ..
            } => {
                let expires_soon = *expires_at <= Self::now() + 300;

                if !expires_soon {
                    return OAuthActions::Refresh {
                        refresh_token: refresh_token.clone().unwrap_or_default(),
                    };
                }

                match refresh_token {
                    Some(token) => OAuthActions::Refresh {
                        refresh_token: token.clone(),
                    },
                    None => OAuthActions::Reconnect,
                }
            }
            OAuthStates::Disconnected { .. } => OAuthActions::Reconnect,
            OAuthStates::Error { .. } => OAuthActions::HandleError,
        }
    }

    fn get_token_from_state(state: &Self::States) -> Option<&str> {
        match state {
            OAuthStates::Connected { access_token, .. } => Some(access_token),
            _ => None,
        }
    }

    fn get_wait_duration(
        state: &Self::States,
        config: &Self::Config,
        context: &mut Self::Context,
    ) -> Duration {
        match state {
            OAuthStates::Init => Duration::from_secs(0),
            OAuthStates::Connected { expires_at, .. } => {
                let time_until_refresh = expires_at.saturating_sub(Self::now() + 300);
                Duration::from_secs(time_until_refresh.max(60))
            }
            OAuthStates::Disconnected => Duration::ZERO,
            OAuthStates::Error { .. } => Duration::from_secs(30),
        }
    }
}

pub type OAuthConnectionHandler = ConnectionHandler<OAuthStrategy>;
