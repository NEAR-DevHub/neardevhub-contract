# NEAR DevHub Contract

## Overview

The smart contract responsible for managing the communities, posts, and permissions made available via the [NEAR DevHub frontend](https://neardevhub.org). The repository for the frontend widgets can be found [here](https://github.com/NEAR-DevHub/neardevhub-bos).

## Getting Started

### Prerequisites

Before starting, make sure you have the following installed:

1. [cargo-near](https://github.com/near/cargo-near), to easily create testnet accounts, build and deploy contracts.
2. [NEAR CLI RS](https://github.com/near/near-cli-rs), to interact with the contract.

## Building

From the root directory, run:

```sh
cd community-factory
cargo near build --no-docker
cd ..
cargo near build --no-docker
```

## Running Tests

From the root directory, run:

```sh
cargo test
```

## Deploying

Using [cargo-near](https://github.com/near/cargo-near), run the following command. Be sure to set your own account id and corresponding network.

```sh
cargo near deploy --no-docker {{account.near}}
cd community-factory
cargo near deploy --no-docker {{community.account.near}}
```
