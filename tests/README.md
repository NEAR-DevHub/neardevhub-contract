
# Integration Test using workspaces-rs

This codebase is designed to deploy and test the Devhub contract on a Sandbox node. 

## Dependencies

- Rust: For writing the test suite.
- `workspaces`: A custom library for handling contract calls.
- `near_units`: For handling NEAR unit conversions.
- `serde_json`: For JSON serialization and deserialization.

## Test Suite

The primary function `test_deploy_contract_self_upgrade` performs the following operations:

### Test Flow:

1. Deploy the `devhub` and `near social` contract on a sandbox.
2. Add various types of posts.
3. Add a community.
4. Upgrade the contract.
5. Get all posts and communities to ensure migration success.

#### Initial Setup

The function `init_contracts` initializes both the `devhub` and `near social` contracts by deploying them to the sandbox.

#### Adding Posts

Posts can be of various types: `Idea`, `Submission`, `Comment`, `Attestation`, and `Sponsorship`. Each is added using the `add_post` function of the contract.

#### Adding a Community

A community is added using the `create_community` function of the contract. The community has attributes like `handle`, `name`, `tag`, `description`, etc.

#### Contract Upgrade

The contract is then self-upgraded using its own compiled WASM code. The test ensures that the upgrade was successful.

#### Migration Check

After upgrading, the code checks whether migration is needed. If yes, it performs the migration.

#### Post Upgrade Verification

Finally, it verifies whether all the data (posts, communities) exist in the upgraded contract as expected.

### How to Run

To run the test, use the following command:

```bash
cargo test
