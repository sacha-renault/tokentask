use oauth2::{
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, IntrospectionUrl,
    PkceCodeChallenge, RedirectUrl, RevocationUrl, Scope, TokenResponse, TokenUrl,
};
use std::time::{Duration, Instant};

use crate::api_connector::{ConnectionHandler, FetchStrategy, TokenError, TokenSuccess};

// OAuth Configuration
#[derive(Debug, Default, Clone, bon::Builder)]
pub struct OAuthConfig {
    token_uri: String,
    auth_uri: String,
    redirect_uri: String,
    client_id: String,
    client_secret: String,
    scope: Option<String>,
}

// OAuth Context tracks refresh token and retry logic
#[derive(Debug)]
pub struct OAuthContext {
    refresh_token: Option<String>,
    consecutive_failures: u32,
    last_attempt: Option<Instant>,
    oauth_client: Client<
        oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
        oauth2::StandardTokenIntrospectionResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        >,
        oauth2::StandardRevocableToken,
        oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
}

pub struct OAuthStrategy;

impl FetchStrategy for OAuthStrategy {
    type Config = OAuthConfig;
    type Context = OAuthContext;

    fn fetch(
        config: &Self::Config,
        context: &mut Self::Context,
    ) -> Result<TokenSuccess, TokenError> {
        println!("Calling fetch");
        context.last_attempt = Some(Instant::now());

        Ok(TokenSuccess {
            token: "token_123".into(),
            duration: Duration::from_secs(30),
        })
        // Err(TokenError {
        //     error_message: "token_123".into(),
        //     duration: Duration::from_secs(30),
        // })
    }

    fn init_context(config: &OAuthConfig) -> Result<Self::Context, ()> {
        let OAuthConfig {
            token_uri,
            auth_uri,
            redirect_uri,
            client_id,
            ..
        } = config.clone();

        let client = oauth2::basic::BasicClient::new(ClientId::new(client_id))
            .set_auth_uri(AuthUrl::new(auth_uri).map_err(|_| ())?)
            .set_token_uri(TokenUrl::new(token_uri).map_err(|_| ())?)
            .set_redirect_uri(RedirectUrl::new(redirect_uri).map_err(|_| ())?);

        Ok(OAuthContext {
            refresh_token: None,
            consecutive_failures: 0,
            last_attempt: None,
            oauth_client: client,
        })
    }
}

pub type OAuthConnectionHandler = ConnectionHandler<OAuthStrategy>;
