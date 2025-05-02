use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use indoc::indoc;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

use astar::heuristic_search::AStarSearch;
use astar::heuristic_search::Heuristic;
use astar::maze_2d::Maze2DAction;
use astar::maze_2d::Maze2DCost;
use astar::maze_2d::Maze2DHeuristicManhattan;
use astar::maze_2d::Maze2DProblem;
use astar::maze_2d::Maze2DSpace;
use astar::maze_2d::Maze2DState;
use astar::space::Action;
use astar::space::Cost;
use astar::space::Problem;
use astar::space::Space;
use astar::space::State;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, env = "LOGS", default_value = "/tmp/logs.org")]
    pub output: PathBuf,

    #[arg()]
    pub problems: Vec<PathBuf>,
}

fn solve<P, H, Sp, St, A, C>(out: &mut BufWriter<dyn std::io::Write>, p: P) -> std::io::Result<()>
where
    P: Problem<Sp, St, A, C>,
    H: Heuristic<P, Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    let mut search = AStarSearch::<P, H, Sp, St, A, C>::new(p);
    writeln!(out, "** A* data\n#+begin_src ron\n{search:?}\n#+end_src")?;

    let path = search.find_first();
    writeln!(out, "*** Path\n#+begin_src ron\n{path:?}\n#+end_src")?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("Logging to {:?}", args.output);

    let file = File::create(&args.output)?;
    let mut out = BufWriter::new(file);

    let maze_str = indoc! {"
      ###
      #S#
      # #
      #G#
      ###
    "};
    writeln!(out, "** Problem")?;
    solve::<
        Maze2DProblem,
        Maze2DHeuristicManhattan,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >(&mut out, Maze2DProblem::try_from(maze_str).unwrap())?;

    for p in &args.problems {
        let space = Maze2DSpace::try_from(p.as_path()).unwrap();
        writeln!(out, "** Space {:?} ({:?})", p, space.dimensions())?;
        writeln!(out, "*** Map")?;
        writeln!(out, "#+begin_quote\n{space}\n#+end_quote")?;
        writeln!(out, "*** Problems")?;
        let mut problem = Maze2DProblem::try_from(p.as_path()).unwrap();

        for instance in 0..10 {
            let mut rng = ChaCha8Rng::seed_from_u64(instance);
            let num_starts = 3;
            let num_goals = 3;
            if problem.randomize(&mut rng, num_starts, num_goals) {
                writeln!(out, "#+begin_quote\n{problem}\n#+end_quote")?;
            } else {
                writeln!(
                    out,
                    "FIXME Failed to generate random problem with seed
            {} with {} starts and {} goals",
                    instance, num_starts, num_goals,
                )?;
            }
        }
    }

    Ok(())
}
