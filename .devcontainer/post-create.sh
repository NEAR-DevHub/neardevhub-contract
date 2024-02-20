#!/bin/bash

(cd discussions && ./build.sh)
(cd community && ./build.sh)
(cd community-factory && ./build.sh)
./build.sh