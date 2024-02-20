#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
