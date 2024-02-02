# NEAR DevHub Contract

## Overview

The smart contract responsible for managing the communities, posts, and permissions made available via the [NEAR DevHub frontend](https://devhub.near.social). The repository for the frontend widgets can be found [here](https://github.com/NEAR-DevHub/neardevhub-bos).

## Getting Started

### Prerequisites

Before starting, make sure you have the following installed:

1. [NEAR CLI RS](https://github.com/near/near-cli-rs), to deploy and interact with the contract.
2. [cargo-near](https://github.com/near/cargo-near), to easily create testnet accounts.

## Building

From the root directory, run:

```cmd
cd community
cargo near build
cd ../community-factory
./build.sh
cd ..
./build.sh
```

## Running Tests

From the root directory, run:

```cmd
cargo test
```

## Deploying

Using [NEAR CLI RS](https://github.com/near/near-cli-rs), run the following command. Be sure to set your own account id and corresponding network.

```cmd
near contract deploy {{account.near}} use-file ./target/wasm32-unknown-unknown/release/devgovgigs.wasm without-init-call network-config {{env}}
near contract deploy {{community.account.near}} use-file ./target/wasm32-unknown-unknown/release/devhub_community_factory.wasm without-init-call network-config {{env}}
```
