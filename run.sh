#!/bin/bash

set -e

cd core
cargo build
cargo run &

cd ../ui
node_modules/electron/dist/electron .