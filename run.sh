#!/bin/sh

pushd decker_util
cargo build --target-dir ../target --release
popd

mkdir -p target/release
cp target/x86_64-unknown-linux-musl/release/decker_util target/release

cargo run --release
