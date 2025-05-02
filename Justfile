build:
	cargo build --all-targets --keep-going

run:
	cargo run  -- data/problems/Maze2D/*.png

test:
	cargo test --all-targets --all-features

report:
	cargo run --bin 'report' --features 'inspect'
	bat $ASTAR_LOGS
