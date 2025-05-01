#![feature(non_null_from_ref)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::ptr::NonNull;

use clap::Parser;
use indoc::indoc;

use astar::debug::type_name;
use astar::heuristic_search::AStarNode;
use astar::heuristic_search::AStarSearch;
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
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, env = "ASTAR_LOGS", default_value = "/tmp/astar_logs.org")]
    pub output: PathBuf,
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

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("Logging to {:?}", args.output);

    let file = File::create(&args.output)?;
    let mut out = BufWriter::new(file);

    writeln!(out, ":PROPERTIES:")?;
    writeln!(out, ":DATE: {}", chrono::offset::Local::now())?;
    writeln!(out, ":END:")?;
    writeln!(out, "#+title: Search library")?;
    writeln!(out)?;
    writeln!(out, "* Data")?;

    writeln!(out, "** Sizes")?;
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
    print_size(&mut out, 1u8)?;
    print_size(&mut out, &args)?;
    print_size(&mut out, args)?;

    writeln!(out, "** Space")?;
    writeln!(out, "*** Maze2D")?;
    writeln!(out, "**** Sizes")?;
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
    print_size(&mut out, Maze2DCell::try_from('#').unwrap())?;
    print_size(&mut out, Maze2DProblemCell::try_from('G').unwrap())?;
    let s0 = Maze2DState { x: 0, y: 0 };
    print_size(&mut out, s0)?;
    let a = Maze2DAction::Up;
    print_size(&mut out, a)?;
    let maze_str = indoc! {"
      ###
      #S#
      # #
      #G#
      ###
    "};

    println!("** Problem");
    let problem = Maze2DProblem::try_from(maze_str).unwrap();
    print_size(&mut out, problem.space())?;
    print_size(&mut out, problem.space().clone())?;
    print_size(&mut out, problem.clone())?;

    writeln!(out, "** Algorithms")?;
    writeln!(out, "*** A*")?;
    writeln!(out, "**** Sizes")?;
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
    print_size(
        &mut out,
        AStarNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(s0, 0, 100),
    )?;
    let h_n = AStarNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(s0, 0, 100);
    print_size(&mut out, h_n.rank())?;
    print_size(
        &mut out,
        AStarNode::new_from_parent(s0, (NonNull::from_ref(&h_n), a), 1, 1),
    )?;
    let mut search = AStarSearch::<
        Maze2DProblem,
        Maze2DHeuristicManhattan,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >::new(problem.clone());
    print_size(&mut out, search.clone())?;
    print_size(&mut out, search.find_first())?;

    writeln!(out, "*** Dijkstra")?;
    writeln!(out, "**** Sizes")?;
    writeln!(out, "| {:60} | {:10} |", "Struct", "Size")?;
    print_size(
        &mut out,
        DijkstraNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(s0, 0),
    )?;
    let h_n = DijkstraNode::<Maze2DState, Maze2DAction, Maze2DCost>::new(s0, 0);
    print_size(&mut out, h_n.rank())?;
    print_size(
        &mut out,
        DijkstraNode::new_from_parent(s0, (NonNull::from_ref(&h_n), a), 1),
    )?;
    let mut search =
        DijkstraSearch::<Maze2DProblem, Maze2DSpace, Maze2DState, Maze2DAction, Maze2DCost>::new(
            problem.clone(),
        );
    print_size(&mut out, search.clone())?;
    print_size(&mut out, search.find_first())?;

    out.flush()?;
    Ok(())
}
