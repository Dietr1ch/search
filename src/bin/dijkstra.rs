use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
use owo_colors::OwoColorize;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::dijkstra::DijkstraSearch;
use search::maze_2d::Maze2DProblem;
use search::maze_2d::Maze2DSpace;
use search::problem::Problem;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(long_version = search::build::CLAP_LONG_VERSION)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        env = "LOGS_DIJKSTRA",
        default_value = "logs/dijkstra.org"
    )]
    pub output: PathBuf,

    #[arg()]
    pub problems: Vec<PathBuf>,

    #[arg(long, default_value_t = 100u64)]
    pub num_instances: u64,
    #[arg(long, default_value_t = 3u16)]
    pub instance_starts: u16,
    #[arg(long, default_value_t = 3u16)]
    pub instance_goals: u16,

    #[command(flatten)]
    color: colorchoice_clap::Color,
}

fn main() -> std::io::Result<()> {
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

        for instance in 0..args.num_instances {
            writeln!(out, "**** Problem {instance}")?;
            let mut rng = ChaCha8Rng::seed_from_u64(instance);
            if let Some(random_problem) =
                p.randomize(&mut rng, args.instance_starts, args.instance_goals)
            {
                writeln!(out, "***** Instance")?;
                writeln!(out, "- Starts:")?;
                let starts = random_problem.starts().clone();
                let goals = random_problem.goals().clone();
                for start in &starts {
                    writeln!(out, "  - {:?}", start)?;
                }
                writeln!(out, "- Goals:")?;
                for goal in &goals {
                    writeln!(out, "  - {:?}", goal)?;
                }
                writeln!(out, "***** Solution")?;
                let search = DijkstraSearch::<_, _, _, _, _>::new(random_problem);
                writeln!(
                    out,
                    "****** Dijkstra run\n#+begin_src ron\n{search:?}\n#+end_src"
                )?;

                for (i, path) in search.take(3).enumerate() {
                    writeln!(out, "******* Path {i} {path}",)?;
                    debug_assert!(starts.contains(&path.start.unwrap()));
                    debug_assert!(goals.contains(&path.end.unwrap()));
                }
            } else {
                writeln!(
                    out,
                    "FIXME Failed to generate random problem with seed
            {} with {} starts and {} goals",
                    instance, args.instance_starts, args.instance_goals,
                )?;
            }
        }
    }

    Ok(())
}
