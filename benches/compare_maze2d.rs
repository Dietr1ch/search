use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use glob::glob;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

use search::heuristic_search::AStarSearch;
use search::maze_2d::Maze2DAction;
use search::maze_2d::Maze2DCost;
use search::maze_2d::Maze2DHeuristicManhattan;
use search::maze_2d::Maze2DProblem;
use search::maze_2d::Maze2DSpace;
use search::maze_2d::Maze2DState;
use search::search::DijkstraSearch;
use search::space::Problem;

fn dijkstra(problem: Maze2DProblem) -> u64 {
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
    for _path in search.take(1) {
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
        let space = Maze2DSpace::try_from(path).unwrap();
        let (x, y) = base_problem.space().dimensions();

        for i in 0..3 {
            let instance_name = format!("{name}[{x}x{y}]:{i}");
            let mut rng = ChaCha8Rng::seed_from_u64(i);

            let num_starts = 3;
            let num_goals = 3;

            if let Some(problem) =
                Maze2DProblem::new_random(&space, &mut rng, num_starts, num_goals)
            {
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
