use tokentask::oauth::{OAuthConfig, OAuthConnectionHandler};

struct Test {
    handler: OAuthConnectionHandler,
    call_api_count: usize,
}

impl Test {
    fn new(config: OAuthConfig) -> Self {
        Self {
            handler: OAuthConnectionHandler::new(config),
            call_api_count: 0,
        }
    }

    fn call_some_api(&mut self, user_id: u64) -> Result<String, ()> {
        let result = self.handler.with_token(|token| {
            self.call_api_count += 1;
            format!("Calling `some_api` with {token} and {user_id}")
        });
        result.ok_or(())
    }
}

fn main() {
    let config = OAuthConfig::builder()
        .client_id("123".into())
        .client_secret("123".into())
        .token_uri("http://localhost.com".into())
        .auth_uri("http://localhost.com".into())
        .redirect_uri("http://localhost.com".into())
        .build();

    let mut handler = Test::new(config);

    let token = handler.call_some_api(2);
    println!("Token: {token:?}");
    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));
    let token = handler.call_some_api(3);
    println!("Token: {token:?}");
}
