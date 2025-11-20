use std::time::{Duration, Instant};

use oauth2::basic::BasicClient;
use oauth2::reqwest::blocking::Client;
use oauth2::url::ParseError;
use oauth2::{ClientId, ClientSecret, Scope, TokenUrl};
use oauth2::{TokenResponse, reqwest};

use crate::api_connector::{ConnectionHandler, FetchStrategy, TokenError, TokenSuccess};

// OAuth Configuration
#[derive(Debug, Clone, bon::Builder)]
pub struct OAuthCredentialsConfig {
    #[builder(with = |s: String| -> Result<_, ParseError> {TokenUrl::new(s)})]
    token_uri: TokenUrl,

    #[builder(with = |s: String| ClientId::new(s))]
    client_id: ClientId,

    #[builder(with = |s: String| ClientSecret::new(s))]
    client_secret: ClientSecret,

    #[builder(with = |s: Vec<String>| s.into_iter().map(|s| Scope::new(s)).collect())]
    #[builder(default = Vec::new())]
    scopes: Vec<Scope>,
}

#[derive(Debug, Clone, bon::Builder)]
pub struct OAuthConfig {
    credentials: OAuthCredentialsConfig,

    /// This will be use if oauth server doesn't provides expiration
    /// Default to one hour
    #[builder(default = Duration::from_secs(3600))]
    default_wait: Duration,

    #[builder(default = 0.1)]
    #[builder(with = |v: f32| -> Result<_, &'static str> { 
    match v {
        v if v <= 0.0 => Err("overlap_percentage must be positive"),
        v if v >= 1.0 => Err("overlap_percentage must be less than 1.0"),
        _ => Ok(v),
    }
})]
    overlap_percentage: f32,
}

// OAuth Context tracks refresh token and retry logic
#[derive(Debug)]
pub struct OAuthContext {
    consecutive_failures: u32,
    last_attempt: Option<Instant>,
    client: Client,
}

fn request_token(
    http_client: &Client,
    config: &OAuthConfig,
) -> Result<
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::RequestTokenError<
        oauth2::HttpClientError<oauth2::reqwest::Error>,
        oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    >,
> {
    let OAuthCredentialsConfig {
        token_uri,
        client_id,
        client_secret,
        scopes,
    } = config.credentials.clone();

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_token_uri(token_uri)
        .exchange_client_credentials()
        .add_scopes(scopes)
        .request(http_client)
}

pub struct OAuthStrategy;

impl FetchStrategy for OAuthStrategy {
    type Config = OAuthConfig;
    type Context = OAuthContext;

    fn fetch(
        config: &Self::Config,
        context: &mut Self::Context,
    ) -> Result<TokenSuccess, TokenError> {
        tracing::debug!("Calling fetch");
        context.last_attempt = Some(Instant::now());

        match request_token(&context.client, config) {
            Ok(resp) => {
                tracing::debug!("{resp:#?}"); // This doesn't print the token: "AccessToken([redacted])"
                context.consecutive_failures = 0;

                let fetch_after = if let Some(exp) = resp.expires_in() {
                    // Refresh 10% before expiration, capped between 5s and 5min
                    let overlap = exp.mul_f32(config.overlap_percentage);
                    let overlap = overlap
                        .min(Duration::from_secs(300))
                        .max(Duration::from_secs(5));
                    exp.saturating_sub(overlap) // Not supposed to saturate since overlap =< 0.1 * exp but just feels more safe so i like it
                } else {
                    config.default_wait
                };

                Ok(TokenSuccess {
                    token: resp.access_token().secret().clone(),
                    fetch_after,
                })
            }
            Err(err) => {
                context.consecutive_failures += 1;
                let retry_after = Duration::from_secs(30);

                Err(TokenError {
                    error_message: err.to_string(),
                    retry_after,
                })
            }
        }
    }

    fn init_context(_config: &OAuthConfig) -> Self::Context {
        OAuthContext {
            consecutive_failures: 0,
            last_attempt: None,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Client should build"),
        }
    }
}

pub type OAuthConnectionHandler = ConnectionHandler<OAuthStrategy>;
