use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
use hrsw::Stopwatch;
use human_duration::human_duration;
use owo_colors::OwoColorize;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::algorithms::dijkstra::DijkstraSearch;
use search::problem::BaseProblem;
use search::problem::ObjectiveProblem;
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

    #[arg(long, default_value_t = 1usize)]
    pub num_solutions: usize,

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
        let mut p = Maze2DProblem::from(space);
        writeln!(out, "**** Base problem")?;
        writeln!(out, "{p}")?;
        writeln!(out, "{p:?}")?;

        for instance in 0..args.num_instances {
            writeln!(out, "**** Problem {instance}")?;
            let mut rng = ChaCha8Rng::seed_from_u64(instance);
            if let Some(random_problem) =
                p.randomize(&mut rng, args.instance_starts, args.instance_goals)
            {
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
                let mut search = DijkstraSearch::new(random_problem);
                writeln!(
                    out,
                    "****** Dijkstra run\n#+begin_src ron\n{search:?}\n#+end_src"
                )?;

                let mut stopwatch = Stopwatch::new_started();
                for i in 0..args.num_solutions {
                    if let Some(path) = search.find_next_goal() {
                        let elapsed = stopwatch.elapsed();
                        debug_assert!(starts.contains(&path.start.unwrap()));
                        debug_assert!(goals.contains(&path.end.unwrap()));
                        writeln!(out, "******* Path {i} {path}",)?;
                        writeln!(out, "Length: {}", path.len())?;
                        writeln!(out, "Elapsed time: {}", human_duration(&elapsed))?;
                        search.write_memory_stats(&mut out)?;
                    } else {
                        break;
                    }
                }
                stopwatch.stop();
                let total_elapsed = stopwatch.elapsed();
                writeln!(out, "******* Total",)?;
                writeln!(out, "Elapsed time: {}", human_duration(&total_elapsed))?;
                search.write_memory_stats(&mut out)?;
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
