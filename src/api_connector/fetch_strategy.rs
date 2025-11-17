pub trait FetchStrategy: Send + Sync + 'static {
    /// Configuration needed to connect/refresh (e.g., client_id, endpoints, credentials)
    type Config: Send + Sync + 'static;

    /// State machine states (e.g., `Init`, `Connected(String)`, `Disconnected`)
    type States: Default;

    /// Actions that trigger state transitions (e.g., `Connect`, `Refresh`, `Reconnect`)
    type Actions;

    /// Establish initial connection - called when state is Init
    ///
    /// Example: Make OAuth request, return `Connected(access_token)`
    fn connect(config: &Self::Config) -> Self::States;

    /// Refresh existing connection - called when token needs renewal
    ///
    /// Example: Use refresh token to get new access token
    fn refresh(config: &Self::Config) -> Self::States;

    /// Execute an action and return the new state
    ///
    /// Example: `Actions::Connect => Self::connect(config)`
    fn execute(config: &Self::Config, action: Self::Actions) -> Self::States;

    /// Decide which action to take based on state transition
    ///
    /// Example: `(_, Init) => Actions::Connect`, `(_, Connected(_)) => Actions::Refresh`
    fn choose_action(previous_state: &Self::States, current_state: &Self::States) -> Self::Actions;

    /// Extract the token from a state, if available
    ///
    /// Example: `Connected(token) => Some(token)`, `_ => None`
    fn get_token_from_state(state: &Self::States) -> Option<&str>;
}
