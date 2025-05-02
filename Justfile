build:
	cargo build --all-targets --keep-going

run:
	cargo run

test:
	cargo test --all-targets --all-features

report:
	cargo run --bin 'report' --features 'inspect'
