#![feature(non_null_from_ref)]

use indoc::indoc;

use astar::heuristic_search::AStarSearch;
use astar::maze_2d::Maze2DAction;
use astar::maze_2d::Maze2DCost;
use astar::maze_2d::Maze2DHeuristicManhattan;
use astar::maze_2d::Maze2DProblem;
use astar::maze_2d::Maze2DSpace;
use astar::maze_2d::Maze2DState;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    let maze_str = indoc! {"
      ###
      #S#
      # #
      #G#
      ###
    "};
    println!("** Problem");
    let problem = Maze2DProblem::try_from(maze_str).unwrap();

    let mut search = AStarSearch::<
        Maze2DProblem,
        Maze2DHeuristicManhattan,
        Maze2DSpace,
        Maze2DState,
        Maze2DAction,
        Maze2DCost,
    >::new(problem);
    println!("** A* data\n#+begin_src ron\n{search:?}\n#+end_src");

    let path = search.find_first();
    println!("*** Path\n#+begin_src ron\n{path:?}\n#+end_src");
}
