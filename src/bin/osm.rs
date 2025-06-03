#[cfg(feature = "osm")]
use std::fs::File;
#[cfg(feature = "osm")]
use std::io::BufWriter;
#[cfg(feature = "osm")]
use std::io::Write;
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
#[cfg(feature = "osm")]
use owo_colors::OwoColorize;

// use search::algorithms::astar::AStarSearch;
// use search::problem::BaseProblem;
// use search::problem::ObjectiveProblem;
// use search::problems::osm::OSMHeuristicDiagonalDistance;
// use search::problems::osm::OSMProblem;
#[cfg(feature = "osm")]
use search::problems::osm::OSMSpace;

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
    #[arg(short, long, env = "LOGS_OSM", default_value = "logs/osm.org")]
    pub output: PathBuf,

    #[arg()]
    pub osm_map: PathBuf,

    // #[arg(long, default_value_t = 100u64)]
    // pub num_instances: u64,
    // #[arg(long, default_value_t = 3u16)]
    // pub instance_starts: u16,
    // #[arg(long, default_value_t = 3u16)]
    // pub instance_goals: u16,

    // #[arg(long, default_value_t = 1usize)]
    // pub num_solutions: usize,
    #[command(flatten)]
    color: colorchoice_clap::Color,
}

#[cfg(not(feature = "osm"))]
fn osm_demo() -> std::io::Result<()> {
    println!("This requires the 'osm' feature.");
    Ok(())
}

#[cfg(feature = "osm")]
fn osm_demo() -> std::io::Result<()> {
    let args = Args::parse();
    args.color.write_global();
    println!("Logging to {:?}", args.output.yellow());

    let file = File::create(&args.output)?;
    let mut out = BufWriter::new(file);

    writeln!(out, "* Runs")?;
    if let Some(space) = OSMSpace::new(&args.osm_map) {
        println!("Loaded map!");
        println!("{space}");
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    #[cfg(feature = "coz_profile")]
    coz::thread_init();

    osm_demo()
}
