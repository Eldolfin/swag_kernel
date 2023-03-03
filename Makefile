all: check run

check:
	cargo check

run:
	cargo run -- -display gtk,zoom-to-fit=on

test:
	cargo test
