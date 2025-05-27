use std::time::Duration;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use glob::glob;
use hrsw::Stopwatch;
use human_duration::human_duration;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::algorithms::astar::AStarSearch;
use search::problem::BaseProblem;
use search::problem::ObjectiveProblem;
use search::problems::maze_2d::Maze2DAction;
use search::problems::maze_2d::Maze2DCost;
use search::problems::maze_2d::Maze2DHeuristicDiagonalDistance;
use search::problems::maze_2d::Maze2DProblem;
use search::problems::maze_2d::Maze2DSpace;
use search::problems::maze_2d::Maze2DState;

const NUM_SOLUTIONS: usize = 2;
/// Maximum time willing to wait for a single benchmark instance.
/// Experiments are carried out at least 5s and at least 100 times, so running a
/// 1s instance takes 1m40s.
const MAX_INSTANCE_TIME: Duration = Duration::from_secs(1);

fn astar(problem: Maze2DProblem) -> u64 {
    let search = AStarSearch::<
        Maze2DHeuristicDiagonalDistance,
        Maze2DProblem,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >::new(problem);

    let mut solutions = 0u64;
    for _path in search.take(NUM_SOLUTIONS) {
        solutions += 1;
    }
    solutions
}

fn compare_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Maze2D Search");

    for path in glob("data/problems/Maze2D/*.png")
        .unwrap()
        .filter_map(std::result::Result::ok)
    {
        let name = path.file_name().unwrap().to_str().unwrap();
        let path: &std::path::Path = path.as_ref();
        let mut base_problem = Maze2DProblem::try_from(path).unwrap();
        let (x, y) = base_problem.space().dimensions();

        for i in 0..5 {
            let instance_name = format!("{name}[{x}x{y}]:{i}");
            let mut rng = ChaCha8Rng::seed_from_u64(i);

            let num_starts = 3;
            let num_goals = 3;

            if let Some(problem) = base_problem.randomize(&mut rng, num_starts, num_goals) {
                let mut astar_search = AStarSearch::<
                    Maze2DHeuristicDiagonalDistance,
                    Maze2DProblem,
                    Maze2DSpace,
                    Maze2DState,
                    Maze2DAction,
                    Maze2DCost,
                >::new(problem.clone());

                let mut astar_solutions = 0;
                let mut astar_stopwatch = Stopwatch::new_started();
                // NOTE: This is just to avoid dropping the search :/
                for _i in 0..NUM_SOLUTIONS {
                    if let Some(path) = astar_search.find_next_goal() {
                        astar_solutions += 1;
                        println!("A* path: {} actions. Path: {}", path.len(), path);
                        astar_search.print_memory_stats();
                    }
                }
                astar_stopwatch.stop();
                let astar_total_elapsed = astar_stopwatch.elapsed();
                if astar_solutions != NUM_SOLUTIONS {
                    astar_search.print_memory_stats();
                }
                if astar_total_elapsed > MAX_INSTANCE_TIME {
                    log::warn!(
                        "Skipping {instance_name} as it takes too long with A* ({})",
                        human_duration(&astar_total_elapsed)
                    );
                    continue;
                }

                group.bench_with_input(BenchmarkId::new("A*", &instance_name), &problem, |b, p| {
                    b.iter(|| astar(p.clone()))
                });
            }
        }
    }
    group.finish();
}

criterion_group!(benches, compare_search);
criterion_main!(benches);
