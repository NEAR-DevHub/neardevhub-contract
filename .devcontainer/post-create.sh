#!/bin/bash

# TODO: update devcontainer later
(cd community && cargo near build)
(cd community-factory && cargo near build)
cargo near build
