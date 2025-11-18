# tokenfetch

Background token refresh without the boilerplate.

## What is this?

You know when you need to keep refreshing OAuth tokens, API keys, or database connections in the background? And you end up writing the same threading + state machine code every time?

This handles that. You define the states and actions, it runs the loop.

## Example

TODO

## TODO

- [ ] Add exponential backoff support
- [ ] Configurable polling interval
- [ ] Metrics/callbacks for state transitions
- [ ] Better error handling
- [ ] Examples for common cases (OAuth, JWT, etc.)