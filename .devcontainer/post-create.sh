#!/bin/bash

(cd community-factory && cargo near build --no-docker)
cargo near build --no-docker
