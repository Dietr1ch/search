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

profile_astar:
	cargo build --bin 'astar' --profile 'bench'
	coz run --- ./target/release/astar --num-instances 100 data/problems/Maze2D/*.png
	coz plot

profile_dijkstra:
	cargo build --bin 'dijkstra' --profile 'bench'
	coz run --- ./target/release/dijkstra --num-instances 100 data/problems/Maze2D/*.png
	coz plot
