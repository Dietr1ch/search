#![feature(non_null_from_ref)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
use indoc::indoc;
use nonmax::NonMaxUsize;
use owo_colors::OwoColorize;

use astar::debug::type_name;
use astar::heuristic_search::AStarNode;
use astar::heuristic_search::AStarSearch;
use astar::maze_2d::Coord;
use astar::maze_2d::Maze2DAction;
use astar::maze_2d::Maze2DCell;
use astar::maze_2d::Maze2DCost;
use astar::maze_2d::Maze2DHeuristicManhattan;
use astar::maze_2d::Maze2DProblem;
use astar::maze_2d::Maze2DProblemCell;
use astar::maze_2d::Maze2DSpace;
use astar::maze_2d::Maze2DState;
use astar::search::DijkstraNode;
use astar::search::DijkstraSearch;
use astar::space::Problem;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(long_version = astar::build::CLAP_LONG_VERSION)]
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

pub fn print_size<T: std::fmt::Debug, W: std::io::Write>(
    out: &mut BufWriter<W>,
    _: T,
) -> std::io::Result<()> {
    use std::mem::size_of;

    let t_type = format!("~{}~", type_name::<T>());
    writeln!(out, "| {:60} | {:10?} |", t_type, size_of::<T>(),)?;
    Ok(())
}

pub fn write_report<W: std::io::Write>(out: &mut BufWriter<W>) -> std::io::Result<()> {
    writeln!(out, ":PROPERTIES:")?;
    writeln!(out, ":END:")?;
    writeln!(out, "#+title: Search library")?;
    writeln!(out)?;
    writeln!(out, "* Data")?;

    writeln!(out, "** Sizes")?;
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
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
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
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
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
    print_size(
        out,
        AStarNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(0usize, s0, 0, 100),
    )?;
    let h_n = AStarNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(0usize, s0, 0, 100);
    print_size(out, h_n.rank())?;
    print_size(
        out,
        AStarNode::new_from_parent(0usize, s0, (NonMaxUsize::new(0usize).unwrap(), a), 1, 1),
    )?;
    let mut search = AStarSearch::<
        Maze2DProblem,
        Maze2DHeuristicManhattan,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >::new(problem.clone());
    print_size(out, search.clone())?;
    print_size(out, search.find_next())?;

    writeln!(out, "*** Dijkstra")?;
    writeln!(out, "**** Sizes")?;
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
    print_size(
        out,
        DijkstraNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(0usize, s0, 0),
    )?;
    let n = DijkstraNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(0usize, s0, 0);
    print_size(out, n.rank())?;
    print_size(
        out,
        DijkstraNode::new_from_parent(0usize, s0, (NonMaxUsize::new(0usize).unwrap(), a), 1),
    )?;
    let mut search =
        DijkstraSearch::<Maze2DProblem, Maze2DSpace, Maze2DState, Maze2DAction, Maze2DCost>::new(
            problem.clone(),
        );
    print_size(out, search.clone())?;
    print_size(out, search.find_next())?;

    out.flush()
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    args.color.write_global();
    println!("Writting report to {:?}", args.type_report.green());

    let file = File::create(&args.type_report)?;
    let mut r = BufWriter::new(file);
    write_report(&mut r)?;

    Ok(())
}
