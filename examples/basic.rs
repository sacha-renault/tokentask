use tokentask::oauth::{OAuthConfig, OAuthConnectionHandler};

fn main() {
    let config = OAuthConfig::builder()
        .client_id("123".into())
        .client_secret("123".into())
        .token_uri("http://localhost.com".into())
        .auth_uri("http://localhost.com".into())
        .redirect_uri("http://localhost.com".into())
        .build();

    let handler = OAuthConnectionHandler::new(config);

    let token = handler.with_token(|token| println!("{token}"));
    println!("Token: {token:?}");
    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));
    let token = handler.with_token(|token| println!("{token}"));
    println!("Token: {token:?}");
}
