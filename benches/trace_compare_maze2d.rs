extern crate std;

use std::path::PathBuf;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::astar::AStarSearch;
use search::dijkstra::DijkstraSearch;
use search::maze_2d::Maze2DAction;
use search::maze_2d::Maze2DCost;
use search::maze_2d::Maze2DHeuristicManhattan;
use search::maze_2d::Maze2DProblem;
use search::maze_2d::Maze2DSpace;
use search::maze_2d::Maze2DState;
use search::problem::Problem;

fn get_instance() -> Maze2DProblem {
    let path = PathBuf::from("data/problems/Maze2D/0.png");
    let instance = 0;
    let instance_starts = 3;
    let instance_goals = 3;

    let mut p = Maze2DProblem::try_from(path.as_path()).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(instance);

    p.randomize(&mut rng, instance_starts, instance_goals)
        .unwrap()
}

fn run_dijkstra(problem: Maze2DProblem) -> u64 {
    let search =
        DijkstraSearch::<Maze2DProblem, Maze2DSpace, Maze2DState, Maze2DAction, Maze2DCost>::new(
            problem,
        );

    let mut solutions = 0u64;
    for _path in search.take(1) {
        solutions += 1;
    }
    solutions
}

fn run_astar(problem: Maze2DProblem) -> u64 {
    let search = AStarSearch::<
        Maze2DHeuristicManhattan,
        Maze2DProblem,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >::new(problem);

    let mut solutions = 0u64;
    for _path in search.take(1) {
        solutions += 1;
    }
    solutions
}

mod iai_wrappers {
    use iai::black_box;

    pub fn iai_trace_baseline() {
        let p = super::get_instance();

        let _ = black_box(p);
    }
    pub fn iai_trace_astar() {
        let p = super::get_instance();

        let num_solutions = super::run_astar(black_box(p));

        let _ = black_box(num_solutions);
    }
    pub fn iai_trace_dijkstra() {
        let p = super::get_instance();

        let num_solutions = super::run_dijkstra(black_box(p));

        let _ = black_box(num_solutions);
    }
}

fn main() {
    let benchmarks: &[&(&'static str, fn())] = &[
        &("iai_trace_baseline", iai_wrappers::iai_trace_baseline),
        &("iai_trace_astar", iai_wrappers::iai_trace_astar),
        &("iai_trace_dijkstra", iai_wrappers::iai_trace_dijkstra),
    ];
    ::iai::runner(benchmarks);
}
