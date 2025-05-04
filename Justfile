build:
	cargo build --all-targets --keep-going

run:
	cargo run  -- data/problems/Maze2D/*.png

test:
	cargo test --all-targets --all-features

report:
	cargo run --bin 'report' --features 'inspect'

bench:
	cargo flamegraph --bench compare_maze2d --output data/benches/criterion/maze2d.svg -- --bench
