/// Report tool
///
/// Generates `docs/type_report.org`
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
use indoc::indoc;
use owo_colors::OwoColorize;

use search::algorithms::astar::AStarHeapNode;
use search::algorithms::astar::AStarRank;
use search::algorithms::astar::AStarSearch;
use search::algorithms::dijkstra::DijkstraHeapNode;
use search::algorithms::dijkstra::DijkstraRank;
use search::algorithms::dijkstra::DijkstraSearch;
use search::debug::type_name;
use search::problem::BaseProblem;
use search::problems::maze_2d::Coord;
use search::problems::maze_2d::Maze2DAction;
use search::problems::maze_2d::Maze2DCell;
use search::problems::maze_2d::Maze2DCost;
use search::problems::maze_2d::Maze2DHeuristicDiagonalDistance;
use search::problems::maze_2d::Maze2DProblem;
use search::problems::maze_2d::Maze2DProblemCell;
use search::problems::maze_2d::Maze2DSpace;
use search::problems::maze_2d::Maze2DState;
use search::search::SearchTreeIndex;
use search::search::SearchTreeNode;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(long_version = search::build::CLAP_LONG_VERSION)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        env = "TYPE_REPORT",
        default_value = "docs/type_report.org"
    )]
    pub type_report: PathBuf,

    #[command(flatten)]
    color: colorchoice_clap::Color,
}

fn print_size<T: std::fmt::Debug, W: std::io::Write>(
    out: &mut BufWriter<W>,
    _: T,
) -> std::io::Result<()> {
    use std::mem::size_of;

    let t_type = format!("~{}~", type_name::<T>());
    let size = size_of::<T>();
    const AVG_CACHELINE_SIZE: usize = 64;
    let items_per_avg_cacheline = AVG_CACHELINE_SIZE / size;
    writeln!(
        out,
        "| {t_type:60} | {size:10?} | {items_per_avg_cacheline:10?} |"
    )?;
    Ok(())
}

fn write_report<W: std::io::Write>(out: &mut BufWriter<W>) -> std::io::Result<()> {
    writeln!(out, ":PROPERTIES:")?;
    writeln!(out, ":VERSION: {:?}", search::build::PKG_VERSION)?;
    writeln!(out, ":GIT_BRANCH: {:?}", shadow_rs::branch())?;
    writeln!(out, ":BUILD_IS_DEBUG: {}", shadow_rs::is_debug())?;
    if search::build::GIT_CLEAN {
        writeln!(out, ":GIT_STATUS: CLEAN")?;
    } else {
        writeln!(out, ":GIT_STATUS: DIRTY")?;
    }
    writeln!(out, ":END:")?;
    writeln!(out, "#+title: Search library")?;
    writeln!(out)?;
    writeln!(out, "* Data")?;

    writeln!(out, "** Sizes")?;
    writeln!(
        out,
        "| {:60} | {:10} | {:10} |",
        "Struct", "Size", "Items/64B"
    )?;
    print_size(out, 1u8)?;
    let buffer = [0u8; 128];
    {
        #[allow(clippy::needless_borrows_for_generic_args)]
        print_size(out, &buffer)?;
    }
    print_size(out, buffer)?;

    writeln!(out, "** Space")?;
    // Maze2D sizes
    writeln!(out, "*** Maze2D")?;
    writeln!(out, "**** Sizes")?;
    writeln!(
        out,
        "| {:60} | {:10} | {:10} |",
        "Struct", "Size", "Items/64B"
    )?;
    print_size(out, Maze2DCell::try_from('#').unwrap())?;
    print_size(out, Maze2DProblemCell::try_from('G').unwrap())?;
    let x = Coord::new(0).unwrap();
    print_size(out, x)?;
    print_size(out, (x, true))?;
    let s0 = Maze2DState::new_from_usize(0, 0).unwrap();
    print_size(out, s0)?;
    let a = Maze2DAction::Up;
    print_size(out, a)?;
    print_size(out, (s0, a))?;

    let maze_str = indoc! {"
      ###
      #S#
      # #
      #G#
      ###
    "};

    writeln!(out, "** Problem")?;
    let problem = Maze2DProblem::try_from(maze_str).unwrap();
    print_size(out, problem.space())?;
    print_size(out, problem.space().clone())?;
    print_size(out, problem.clone())?;

    writeln!(out, "** Algorithms")?;
    writeln!(out, "*** A*")?;
    writeln!(out, "**** Sizes")?;
    writeln!(
        out,
        "| {:60} | {:10} | {:10} |",
        "Struct", "Size", "Items/64B"
    )?;
    // search_tree: Arena<SearchTreeNode>
    let node = SearchTreeNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(0usize, s0, None, 0);
    print_size(out, node)?;
    // heap: Vec<(Rank, SearchTreeIndex)>
    print_size(
        out,
        AStarHeapNode {
            rank: AStarRank::new(0, 0),
            node_index: SearchTreeIndex::fake_new(),
        },
    )?;
    // node_map: HashMap<State, SearchTreeIndex>
    print_size(out, SearchTreeIndex::fake_new())?;
    print_size(out, (s0, SearchTreeIndex::fake_new()))?;
    let mut search = AStarSearch::<
        Maze2DHeuristicDiagonalDistance,
        Maze2DProblem,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >::new(problem.clone());
    print_size(out, search.find_next_goal())?;
    print_size(out, search)?;

    writeln!(out, "*** Dijkstra")?;
    writeln!(out, "**** Sizes")?;
    writeln!(
        out,
        "| {:60} | {:10} | {:10} |",
        "Struct", "Size", "Items/64B"
    )?;
    // search_tree: Arena<SearchTreeNode>
    let node = SearchTreeNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(0usize, s0, None, 0);
    print_size(out, node)?;
    // heap: Vec<(Rank, SearchTreeIndex)>
    print_size(
        out,
        DijkstraHeapNode {
            rank: DijkstraRank::new(0),
            node_index: SearchTreeIndex::fake_new(),
        },
    )?;
    // node_map: HashMap<State, SearchTreeIndex>
    print_size(out, SearchTreeIndex::fake_new())?;
    print_size(out, (s0, SearchTreeIndex::fake_new()))?;
    let mut search =
        DijkstraSearch::<Maze2DProblem, Maze2DSpace, Maze2DState, Maze2DAction, Maze2DCost>::new(
            problem.clone(),
        );
    print_size(out, search.find_next_goal())?;
    print_size(out, search)?;

    out.flush()
}

fn main() -> std::io::Result<()> {
    #[cfg(feature = "coz_profile")]
    coz::thread_init();

    let args = Args::parse();
    args.color.write_global();
    println!("Writing report to {:?}", args.type_report.green());

    let file = File::create(&args.type_report)?;
    let mut r = BufWriter::new(file);
    write_report(&mut r)?;

    Ok(())
}
