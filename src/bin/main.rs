use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
use owo_colors::OwoColorize;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::algorithms::astar::AStarSearch;
use search::problem::BaseProblem;
use search::problem::ObjectiveProblem;
use search::problems::maze_2d::Maze2DHeuristicDiagonalDistance;
use search::problems::maze_2d::Maze2DProblem;
use search::problems::maze_2d::Maze2DSpace;

#[cfg(feature = "mem_profile")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
#[cfg(not(feature = "mem_profile"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(long_version = search::build::CLAP_LONG_VERSION)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, env = "LOGS", default_value = "logs/main.org")]
    pub output: PathBuf,

    #[arg()]
    pub problems: Vec<PathBuf>,

    #[command(flatten)]
    color: colorchoice_clap::Color,
}

fn main() -> std::io::Result<()> {
    #[cfg(feature = "coz_profile")]
    coz::thread_init();

    let args = Args::parse();
    args.color.write_global();
    println!("Logging to {:?}", args.output.yellow());

    let file = File::create(&args.output)?;
    let mut out = BufWriter::new(file);

    writeln!(out, "* Runs")?;
    for p in &args.problems {
        let space = Maze2DSpace::try_from(p.as_path()).unwrap();
        writeln!(out, "** Space {:?} ({:?})", p, space.dimensions())?;
        writeln!(out, "*** Map")?;
        writeln!(out, "#+begin_quote\n{space}\n#+end_quote")?;
        writeln!(out, "*** Problems")?;
        let mut p = Maze2DProblem::try_from(p.as_path()).unwrap();

        for instance in 0..10 {
            writeln!(out, "**** Problem {instance}")?;
            let mut rng = ChaCha8Rng::seed_from_u64(instance);
            let num_starts = 3;
            let num_goals = 3;
            if let Some(random_problem) = p.randomize(&mut rng, num_starts, num_goals) {
                writeln!(out, "***** Instance")?;
                writeln!(out, "- Starts:")?;
                let starts = random_problem.starts().to_vec();
                let goals = random_problem.goals().to_vec();
                for start in &starts {
                    writeln!(out, "  - {start:?}")?;
                }
                writeln!(out, "- Goals:")?;
                for goal in &goals {
                    writeln!(out, "  - {goal:?}")?;
                }
                writeln!(out, "***** Solution")?;
                let search = AStarSearch::<Maze2DHeuristicDiagonalDistance, _, _, _, _, _>::new(
                    random_problem,
                );
                writeln!(out, "****** A* run\n#+begin_src ron\n{search:?}\n#+end_src")?;

                for (i, path) in search.take(3).enumerate() {
                    writeln!(
                        out,
                        "******* Path {i} {path}\n#+begin_src ron\n{path:?}\n#+end_src",
                    )?;
                    debug_assert!(starts.contains(&path.start.unwrap()));
                    debug_assert!(goals.contains(&path.end.unwrap()));
                }
            } else {
                writeln!(
                    out,
                    "FIXME Failed to generate random problem with seed
            {instance} with {num_starts} starts and {num_goals} goals",
                )?;
            }
        }
    }

    Ok(())
}
