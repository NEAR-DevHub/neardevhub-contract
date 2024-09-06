#!/bin/bash

# TODO: update devcontainer later
(cd discussions && cargo near build)
(cd community && cargo near build)
(cd community-factory && cargo near build)
cargo near build
