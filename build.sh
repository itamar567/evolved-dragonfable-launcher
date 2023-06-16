#!/bin/bash

set -e

cd ui
npm run build-linux
npm run build-win
( cd out/evolved-dragonfable-launcher-linux-x64 && mv evolved-dragonfable-launcher ui )
( cd out/evolved-dragonfable-launcher-win32-x64 && mv evolved-dragonfable-launcher.exe ui )

cd ../core
cargo build --release
cross build --release --target x86_64-pc-windows-gnu
cp target/release/evolved-dragonfable-launcher ../ui/out/evolved-dragonfable-launcher-linux-x64/
cp target/x86_64-pc-windows-gnu/release/evolved-dragonfable-launcher.exe ../ui/out/evolved-dragonfable-launcher-win32-x64/

cd ../ui/out/
( cd evolved-dragonfable-launcher-linux-x64 && zip ../evolved-dragonfable-launcher-linux.zip -r * )
( cd evolved-dragonfable-launcher-win32-x64 && zip ../evolved-dragonfable-launcher-windows.zip -r * )

[ ! -d "../../out" ] && mkdir ../../out

cp evolved-dragonfable-launcher-linux.zip ../../out/
cp evolved-dragonfable-launcher-windows.zip ../../out/
