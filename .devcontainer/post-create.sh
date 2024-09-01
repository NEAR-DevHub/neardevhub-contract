#!/bin/bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.sh | sh
(cd discussions && cargo near build --no-docker)
(cd community && cargo near build --no-docker)
(cd community-factory && cargo near build --no-docker)
cargo near build --no-docker
