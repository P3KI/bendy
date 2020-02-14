# just is a cross-platform command runner:
# https://github.com/casey/just

default: test

# install the minimum version of rust supported by bendy
install-minimum-supported-rust:
	rustup install 1.36.0

# test all combinations of features, as well as the minimum supported version of rust
test:
	rm -f Cargo.lock
	cargo test --all
	cargo test --all --no-default-features
	cargo test --all --features serde
	rm -f Cargo.lock
	cargo +1.36.0 test --all
	cargo +1.36.0 test --all --no-default-features
	cargo +1.36.0 test --all --features serde
