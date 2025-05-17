use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use glob::glob;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::algorithms::astar::AStarSearch;
use search::algorithms::dijkstra::DijkstraSearch;
use search::problem::BaseProblem;
use search::problem::ObjectiveProblem;
use search::problems::maze_2d::Maze2DAction;
use search::problems::maze_2d::Maze2DCost;
use search::problems::maze_2d::Maze2DHeuristicManhattan;
use search::problems::maze_2d::Maze2DProblem;
use search::problems::maze_2d::Maze2DSpace;
use search::problems::maze_2d::Maze2DState;

const NUM_SOLUTIONS: usize = 2;

fn dijkstra(problem: Maze2DProblem) -> u64 {
    let search =
        DijkstraSearch::<Maze2DProblem, Maze2DSpace, Maze2DState, Maze2DAction, Maze2DCost>::new(
            problem,
        );

    let mut solutions = 0u64;
    for _path in search.take(NUM_SOLUTIONS) {
        solutions += 1;
    }
    solutions
}

fn astar(problem: Maze2DProblem) -> u64 {
    let search = AStarSearch::<
        Maze2DHeuristicManhattan,
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
                    Maze2DHeuristicManhattan,
                    Maze2DProblem,
                    Maze2DSpace,
                    Maze2DState,
                    Maze2DAction,
                    Maze2DCost,
                >::new(problem.clone());

                let mut solutions = 0;
                // NOTE: This is just to avoid dropping the search :/
                for _i in 0..NUM_SOLUTIONS {
                    if let Some(path) = astar_search.find_next_goal() {
                        solutions += 1;
                        println!("A* path: {} actions. Path: {}", path.len(), path);
                    }
                }
                astar_search.print_memory_stats();

                if solutions > 0 {
                    let mut dijkstra_search = DijkstraSearch::<
                        Maze2DProblem,
                        Maze2DSpace,
                        Maze2DState,
                        Maze2DAction,
                        Maze2DCost,
                    >::new(problem.clone());

                    // NOTE: This is just to avoid dropping the search :/
                    for _i in 0..NUM_SOLUTIONS {
                        if let Some(path) = dijkstra_search.find_next_goal() {
                            println!("Dijkstra path: {} actions. Path: {}", path.len(), path);
                        }
                    }
                    dijkstra_search.print_memory_stats();
                }

                group.bench_with_input(BenchmarkId::new("A*", &instance_name), &problem, |b, p| {
                    b.iter(|| astar(p.clone()))
                });
                group.bench_with_input(
                    BenchmarkId::new("Dijkstra", &instance_name),
                    &problem,
                    |b, p| b.iter(|| dijkstra(p.clone())),
                );
            }
        }
    }
    group.finish();
}

criterion_group!(benches, compare_search);
criterion_main!(benches);
