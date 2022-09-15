#!/bin/sh

platform=$(uname)
echo $platform

pushd decker_util

if [[ $platform == 'Darwin' ]]; then
    RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" cargo build  --release
else
    cargo build  --release
fi

popd

mkdir -p target/release
cp decker_util/target/x86_64-unknown-linux-musl/release/decker_util target/release

cargo run --release
