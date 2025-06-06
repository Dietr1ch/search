build:
	cargo build \
	  --all-targets \
	  --all-features \
	  --keep-going

run:
	cargo run \
	  -- \
	  data/problems/Maze2D/*.png

test:
	cargo test \
	  --all-targets \
	  --all-features

report:
	cargo run \
	  --bin 'report' \
	  --profile 'release' \
	  --features 'inspect'

bench:
	mkdir -p data/benches/criterion/
	cargo flamegraph \
	  --bench sample_compare_maze2d \
	  --output data/benches/criterion/maze2d.svg \
	  -- \
	  --bench

clean_bench:
	rm -rf \
	  data/benches/criterion \
	  target/criterion

profile_astar:
	cargo build \
	  --bin 'astar' \
	  --profile 'bench' \
	  --features 'coz_profile'
	coz run \
	  --source-scope src/% \
	  --output data/profiles/astar/profile.coz \
	  --- \
	  ./target/release/astar \
	  --num-instances 10000 \
	  data/problems/Maze2D/*.png
	coz plot

profile_dijkstra:
	cargo build \
	  --bin 'dijkstra' \
	  --profile 'bench' \
	  --features 'coz_profile'
	coz run \
	  --source-scope src/% \
	  --output data/profiles/dijkstra/profile.coz \
	  --- \
	  ./target/release/dijkstra \
	  --num-instances 100 \
	  data/problems/Maze2D/*.png
	coz plot

profile_trace_maze2d:
	cargo bench \
	  trace_compare_maze2d
