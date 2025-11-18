use std::time::Duration;

pub trait FetchStrategy: Send + Sync + 'static {
    /// Configuration needed to connect/refresh (e.g., client_id, endpoints, credentials)
    type Config: Send + Sync + 'static;

    /// State machine states (e.g., `Init`, `Connected(String)`, `Disconnected`)
    type States: Default;

    /// Actions that trigger state transitions (e.g., `Connect`, `Refresh`, `Reconnect`)
    type Actions;

    /// Use to store state during the strategy
    type Context: Default;

    /// Execute an action and return the new state
    ///
    /// Example: `Actions::Connect => Self::connect(config)`
    fn execute(
        config: &Self::Config,
        action: Self::Actions,
        context: &mut Self::Context,
    ) -> Self::States;

    /// Decide which action to take based on state transition
    ///
    /// Example: `(_, Init) => Actions::Connect`, `(_, Connected(_)) => Actions::Refresh`
    fn choose_action(state: &Self::States, context: &mut Self::Context) -> Self::Actions;

    /// Extract the token from a state, if available
    ///
    /// Example: `Connected(token) => Some(token)`, `_ => None`
    fn get_token_from_state(state: &Self::States) -> Option<&str>;

    /// Get the next duration
    fn get_wait_duration(
        state: &Self::States,
        config: &Self::Config,
        context: &mut Self::Context,
    ) -> Duration;
}
