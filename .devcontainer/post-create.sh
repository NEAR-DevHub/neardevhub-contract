#!/bin/bash

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/near-cli-rs/releases/latest/download/near-cli-rs-installer.sh | sh

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.sh | sh
(cd discussions && cargo near build --no-docker)
(cd community && cargo near build --no-docker)
(cd community-factory && cargo near build --no-docker)
cargo near build --no-docker
