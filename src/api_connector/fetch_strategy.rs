use std::time::Duration;

pub struct TokenSuccess {
    pub token: String,
    pub duration: Duration,
}

pub struct RetryDuration(pub Duration);

pub trait FetchStrategy: Send + Sync + 'static {
    type Config: Send + Sync + 'static;
    type Context;

    fn fetch(
        config: &Self::Config,
        context: &mut Self::Context,
    ) -> Result<TokenSuccess, RetryDuration>;

    fn init_context(config: &Self::Config) -> Result<Self::Context, ()>;
}
