use tokentask::{
    FetchBehavior,
    oauth::{OAuthConfig, OAuthConnectionHandler, OAuthCredentialsConfig},
};

struct Test {
    handler: OAuthConnectionHandler,
    call_api_count: usize,
}

impl Test {
    fn new(config: OAuthConfig) -> Self {
        Self {
            handler: OAuthConnectionHandler::new(FetchBehavior::OldTokenRemainsValid, config),
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
    let credentials = OAuthCredentialsConfig::builder()
        .client_id("123".into())
        .client_secret("123".into())
        .token_uri("http://localhost.com".into())
        .unwrap()
        .build();

    let config = OAuthConfig::builder().credentials(credentials).build();

    let mut handler = Test::new(config);

    let token = handler.call_some_api(2);
    println!("Token: {token:?}");
    std::thread::sleep(std::time::Duration::from_secs(31));
    let token = handler.call_some_api(3);
    println!("Token: {token:?}");
}
