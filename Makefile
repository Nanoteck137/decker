debug ?= 0
ifeq ($(debug), 1)
	EXTRA_FLAG=
	MODE=debug
else
	EXTRA_FLAG=--release
	MODE=release
endif

DECKER_UTIL_EXE=target/x86_64-unknown-linux-musl/$(MODE)/decker_util

all: decker

install: decker_util
	cargo install --path .

build: decker_util
	cargo build $(EXTRA_FLAG)

run: decker_util
	cargo run $(EXTRA_FLAG) -- $(ARGS)

decker_util:
	RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" cargo build --target=x86_64-unknown-linux-musl --manifest-path decker_util/Cargo.toml --target-dir target/ $(EXTRA_FLAG)
	cp $(DECKER_UTIL_EXE) target/decker_util


.PHONY: decker_util

