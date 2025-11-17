mod api_connector;
mod oauth;
mod utils;

use oauth::{OAuthConfig, OAuthConnectionHandler};

fn main() {
    let config = OAuthConfig::builder()
        .client_id("123".into())
        .client_secret("123".into())
        .token_url("123".into())
        .build();

    let handler = OAuthConnectionHandler::new(config);

    let token = handler.get_token();
    println!("Token: {token:?}");
    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));
    let token = handler.get_token();
    println!("Token: {token:?}");
}
