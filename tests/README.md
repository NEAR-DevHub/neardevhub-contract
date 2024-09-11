# Integration Test using near-workspaces-rs

This codebase is designed to deploy and test the Devhub contract on a Sandbox node.

## Dependencies

- Rust: For writing the test suite.
- `near-workspaces`: A custom library for handling contract calls.
- `near_units`: For handling NEAR unit conversions.
- `serde_json`: For JSON serialization and deserialization.

### How to Run

To run the test, use the following command:

```bash
cargo test
```

NOTE: integration tests automatically build the `devhub-community-factory` and  
`devhub` they depend on.
