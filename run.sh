#!/bin/sh

pushd decker_util
cargo build --release
popd

mkdir -p target/release
cp decker_util/target/x86_64-unknown-linux-musl/release/decker_util target/release

cargo run --release
