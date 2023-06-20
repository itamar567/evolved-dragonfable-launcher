#!/bin/bash

set -e

pkill evolved-dragonf || true

cd core
cargo build
cargo run &

cd ../ui
node_modules/electron/dist/electron .