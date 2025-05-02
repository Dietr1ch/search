use rustc_hash::FxHashSet;
use std::hash::Hash;

use num_traits::identities::one;
use num_traits::identities::zero;

use crate::heuristic_search::Heuristic;
use crate::space::Action;
use crate::space::Cost;
use crate::space::Problem;
use crate::space::Space;
use crate::space::State;

type Coord = u32;

const MAX_ELEMENTS_DISPLAYED: usize = 20;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Maze2DState {
    pub x: Coord,
    pub y: Coord,
}
impl State for Maze2DState {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Maze2DAction {
    Up,        // y++
    Down,      // y--
    Left,      // x--
    Right,     // x++
    LeftUp,    // x--, y++
    RightUp,   // x++, y++
    LeftDown,  // x--, y--
    RightDown, // x++, y--
}
impl Action for Maze2DAction {}

pub type Maze2DCost = u32;
impl Cost for Maze2DCost {}

use derive_more::Display;
#[derive(Copy, Clone, Debug, Display, PartialEq)]
pub enum Maze2DCell {
    #[display("░")]
    Empty,
    #[display("█")]
    Wall,
}

use thiserror::Error;
#[derive(Debug, Error)]
pub enum Maze2DCellParseError {
    #[error("Invalid character '{0}' found.")]
    InvalidCharacter(char),
}

impl std::convert::TryFrom<char> for Maze2DCell {
    type Error = Maze2DCellParseError;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            ' ' | '.' => Ok(Maze2DCell::Empty),
            '#' | '█' => Ok(Maze2DCell::Wall),
            ch => Err(Maze2DCellParseError::InvalidCharacter(ch)),
        }
    }
}

#[derive(Clone)]
pub struct Maze2DSpace {
    pub map: Vec<Vec<Maze2DCell>>,
}

impl Maze2DSpace {
    pub fn new_from_map(map: Vec<Vec<Maze2DCell>>) -> Self {
        Self { map }
    }
    pub fn new_empty_with_dimensions(x: usize, y: usize) -> Self {
        Self {
            map: vec![vec![Maze2DCell::Empty; x]; y],
        }
    }

    pub fn dimensions(&self) -> (Coord, Coord) {
        if self.map.is_empty() {
            return (0, 0);
        }
        (self.map[0].len() as Coord, self.map.len() as Coord)
    }
    #[inline(always)]
    fn at(&self, state: &Maze2DState) -> Maze2DCell {
        debug_assert!(self.valid(state));
        unsafe {
            *self
                .map
                .get_unchecked(state.y as usize)
                .get_unchecked(state.x as usize)
        }
    }

    pub fn supports_random_state() -> bool {
        true
    }
    pub fn random_state<R: rand::Rng>(&self, r: &mut R) -> Option<Maze2DState> {
        let (max_x, max_y) = self.dimensions();

        for _tries in 0..1000 {
            let x: Coord = r.random::<Coord>() % max_x;
            let y: Coord = r.random::<Coord>() % max_y;
            let cell: Maze2DCell = self.map[y as usize][x as usize];
            match cell {
                Maze2DCell::Empty => {
                    return Some(Maze2DState { x, y });
                }
                _ => {}
            }
        }

        None
    }
}

impl Space<Maze2DState, Maze2DAction, Maze2DCost> for Maze2DSpace {
    #[inline(always)]
    fn apply(&self, state: &Maze2DState, action: &Maze2DAction) -> Option<Maze2DState> {
        match action {
            Maze2DAction::Up => Some(Maze2DState {
                x: state.x,
                y: state.y + 1,
            }),
            Maze2DAction::Down => Some(Maze2DState {
                x: state.x,
                y: state.y - 1,
            }),
            Maze2DAction::Left => Some(Maze2DState {
                x: state.x - 1,
                y: state.y,
            }),
            Maze2DAction::Right => Some(Maze2DState {
                x: state.x + 1,
                y: state.y,
            }),
            Maze2DAction::LeftUp => Some(Maze2DState {
                x: state.x - 1,
                y: state.y + 1,
            }),
            Maze2DAction::RightUp => Some(Maze2DState {
                x: state.x + 1,
                y: state.y + 1,
            }),
            Maze2DAction::LeftDown => Some(Maze2DState {
                x: state.x - 1,
                y: state.y + 1,
            }),
            Maze2DAction::RightDown => Some(Maze2DState {
                x: state.x + 1,
                y: state.y - 1,
            }),
        }
    }

    #[inline(always)]
    fn valid(&self, state: &Maze2DState) -> bool {
        let (max_x, max_y) = self.dimensions();
        state.x < max_x && state.y < max_y
    }

    /// Gets the neighbours of a given position.
    ///
    /// NOTE: These states can only be used with the current Maze
    fn neighbours(&self, state: &Maze2DState) -> Vec<(Maze2DState, Maze2DAction)> {
        let mut v = Vec::<(Maze2DState, Maze2DAction)>::new();
        let (max_x, max_y) = self.dimensions();

        let prev = Coord::MAX;
        let same = zero::<Coord>();
        let next = one::<Coord>();

        for (dx, dy, action) in [
            // Left
            (prev, prev, Maze2DAction::LeftDown),
            (prev, same, Maze2DAction::Left),
            (prev, next, Maze2DAction::LeftUp),
            // Center
            (same, prev, Maze2DAction::Down),
            // (same, same),
            (same, next, Maze2DAction::Up),
            // Right
            (next, prev, Maze2DAction::RightDown),
            (next, same, Maze2DAction::Right),
            (next, next, Maze2DAction::RightUp),
        ] {
            let new_x: Coord = state.x.wrapping_add(dx);
            let new_y: Coord = state.y.wrapping_add(dy);
            if new_x < max_x && new_y < max_y {
                let s = Maze2DState { x: new_x, y: new_y };
                debug_assert!(self.valid(&s));
                if self.at(&s) != Maze2DCell::Wall {
                    v.push((s, action));
                }
            }
        }
        v
    }
}

impl std::fmt::Display for Maze2DSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let d = self.dimensions();
        writeln!(f, "Maze2D({}x{}):", d.0, d.1)?;
        for line in self.map.iter().take(MAX_ELEMENTS_DISPLAYED) {
            for cell in line.iter().take(MAX_ELEMENTS_DISPLAYED) {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for Maze2DSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Maze2D{:?}", self.dimensions())
    }
}

#[derive(Debug, Error)]
pub enum Maze2DSpaceParseError {
    #[error("Invalid image '{p}'")]
    InvalidImage { p: std::path::PathBuf },
    #[error("I/O error when loading '{p}': {e}")]
    IOError {
        p: std::path::PathBuf,
        e: std::io::Error,
    },
    #[error("Image error when loading '{p}': {e}")]
    ImageError {
        p: std::path::PathBuf,
        e: image::ImageError,
    },
}

impl std::convert::TryFrom<&std::path::Path> for Maze2DSpace {
    type Error = Maze2DSpaceParseError;

    fn try_from(p: &std::path::Path) -> Result<Self, Self::Error> {
        use image::ImageReader;

        let img = ImageReader::open(p)
            .map_err(|e| Maze2DSpaceParseError::IOError {
                p: p.to_path_buf(),
                e,
            })?
            .decode()
            .map_err(|e| Maze2DSpaceParseError::ImageError {
                p: p.to_path_buf(),
                e,
            })?
            .grayscale()
            .into_rgb8();

        let max_x = img.width() as usize;
        let max_y = img.height() as usize;
        let mut space = Maze2DSpace::new_empty_with_dimensions(max_x, max_y);

        for y in 0..img.height() {
            for x in 0..img.width() {
                let px = img.get_pixel(x, y);
                space.map[y as usize][x as usize] = match px.0 {
                    [u8::MIN, u8::MIN, u8::MIN] => Maze2DCell::Wall,
                    [u8::MAX, u8::MAX, u8::MAX] => Maze2DCell::Empty,
                    _ => Maze2DCell::Empty,
                }
            }
        }

        Ok(space)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct Maze2DProblem {
    space: Maze2DSpace,
    starts: Vec<Maze2DState>,
    goals: FxHashSet<Maze2DState>,
}

impl Problem<Maze2DSpace, Maze2DState, Maze2DAction, Maze2DCost> for Maze2DProblem {
    fn space(&self) -> &Maze2DSpace {
        &self.space
    }
    fn starts(&self) -> &Vec<Maze2DState> {
        &self.starts
    }
    fn goals(&self) -> &FxHashSet<Maze2DState> {
        &self.goals
    }

    fn randomize<R: rand::Rng>(
        &mut self,
        r: &mut R,
        num_starts: u16,
        num_goals: u16,
    ) -> Option<Maze2DProblem> {
        let mut starts = vec![];
        let mut goals = FxHashSet::<Maze2DState>::default();
        const MAX_RANDOM_STATE_TRIES: usize = 1000;

        for _tries in 0..MAX_RANDOM_STATE_TRIES {
            if let Some(random_state) = self.space().random_state::<R>(r) {
                if starts.len() < num_starts as usize {
                    starts.push(random_state);
                } else if goals.len() < num_goals as usize {
                    goals.insert(random_state);
                } else {
                    return Some(Maze2DProblem {
                        space: self.space.clone(),
                        starts,
                        goals,
                    });
                }
            }
        }

        None
    }
}

#[derive(Copy, Clone, Debug, Display, PartialEq)]
pub enum Maze2DProblemCell {
    Cell(Maze2DCell),
    #[display("S")]
    Start,
    #[display("G")]
    Goal,
}

#[derive(Debug, Error)]
pub enum Maze2DProblemCellParseError {
    #[error("Invalid cell {e}")]
    InvalidCell { e: Maze2DCellParseError },
}

impl std::convert::TryFrom<char> for Maze2DProblemCell {
    type Error = Maze2DProblemCellParseError;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            'S' => Ok(Maze2DProblemCell::Start),
            'G' => Ok(Maze2DProblemCell::Goal),
            ch => {
                let cell = Maze2DCell::try_from(ch)
                    .map_err(|e| Maze2DProblemCellParseError::InvalidCell { e })?;
                Ok(Maze2DProblemCell::Cell(cell))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum Maze2DProblemParseError {
    #[error("Empty input")]
    EmptyInput,
    #[error("Invalid cell {e} found at ({x},{y})")]
    InvalidCell {
        e: Maze2DProblemCellParseError,
        x: usize,
        y: usize,
    },
    #[error("I/O error when loading '{p}': {e}")]
    IOError {
        p: std::path::PathBuf,
        e: std::io::Error,
    },
    #[error("Image error when loading '{p}': {e}")]
    ImageError {
        p: std::path::PathBuf,
        e: image::ImageError,
    },
}

impl std::convert::TryFrom<&str> for Maze2DProblem {
    type Error = Maze2DProblemParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let lines: Vec<&str> = s.lines().collect();

        if lines.is_empty() {
            return Err(Maze2DProblemParseError::EmptyInput);
        }
        if lines[0].is_empty() {
            return Err(Maze2DProblemParseError::EmptyInput);
        }

        let max_x = lines[0].len();
        let max_y = lines.len();
        let mut problem = Maze2DProblem {
            space: Maze2DSpace::new_empty_with_dimensions(max_x, max_y),
            starts: vec![],
            goals: FxHashSet::default(),
        };

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let cell = Maze2DProblemCell::try_from(ch)
                    .map_err(|e| Maze2DProblemParseError::InvalidCell { e, x, y })?;

                problem.space.map[y][x] = match cell {
                    Maze2DProblemCell::Start => {
                        problem.starts.push(Maze2DState {
                            x: x as Coord,
                            y: y as Coord,
                        });
                        Maze2DCell::Empty
                    }
                    Maze2DProblemCell::Goal => {
                        problem.goals.insert(Maze2DState {
                            x: x as Coord,
                            y: y as Coord,
                        });
                        Maze2DCell::Empty
                    }
                    Maze2DProblemCell::Cell(c) => c,
                }
            }
        }

        Ok(problem)
    }
}

impl std::convert::TryFrom<&std::path::Path> for Maze2DProblem {
    type Error = Maze2DProblemParseError;

    fn try_from(p: &std::path::Path) -> Result<Self, Self::Error> {
        use image::ImageReader;

        let img = ImageReader::open(p)
            .map_err(|e| Maze2DProblemParseError::IOError {
                p: p.to_path_buf(),
                e,
            })?
            .decode()
            .map_err(|e| Maze2DProblemParseError::ImageError {
                p: p.to_path_buf(),
                e,
            })?
            .grayscale()
            .into_rgb8();

        let max_x = img.width() as usize;
        let max_y = img.height() as usize;
        let mut p = Maze2DProblem {
            space: Maze2DSpace::new_empty_with_dimensions(max_x, max_y),
            starts: vec![],
            goals: FxHashSet::<Maze2DState>::default(),
        };

        for y in 0..img.height() {
            for x in 0..img.width() {
                let px = img.get_pixel(x, y);
                p.space.map[y as usize][x as usize] = match px.0 {
                    [u8::MIN, u8::MIN, u8::MIN] => Maze2DCell::Empty,
                    [u8::MAX, u8::MAX, u8::MAX] => Maze2DCell::Wall,
                    [u8::MIN, u8::MAX, u8::MIN] => {
                        // GREEN (goal)
                        p.goals.insert(Maze2DState { x, y });
                        Maze2DCell::Empty
                    }
                    [u8::MIN, u8::MIN, u8::MAX] => {
                        // BLUE (start)
                        p.starts.push(Maze2DState { x, y });
                        Maze2DCell::Empty
                    }
                    _ => Maze2DCell::Empty,
                }
            }
        }

        Ok(p)
    }
}

impl std::fmt::Display for Maze2DProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let d = self.space.dimensions();
        writeln!(
            f,
            "Maze2DProblem({}x{}) (s:{:?}, g:{:?}):",
            d.0, d.1, self.starts, self.goals
        )?;
        for (y, line) in self
            .space
            .map
            .iter()
            .enumerate()
            .take(MAX_ELEMENTS_DISPLAYED)
        {
            for (x, cell) in line.iter().enumerate().take(MAX_ELEMENTS_DISPLAYED) {
                let is_start = self.starts.contains(&Maze2DState {
                    x: x as Coord,
                    y: y as Coord,
                });
                let is_goal = self.goals.contains(&Maze2DState {
                    x: x as Coord,
                    y: y as Coord,
                });

                match (is_start, is_goal) {
                    (true, true) => {
                        write!(f, "!")?;
                    }
                    (true, false) => {
                        write!(f, "S")?;
                    }
                    (false, true) => {
                        write!(f, "G")?;
                    }
                    (false, false) => {
                        write!(f, "{}", cell)?;
                    }
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct Maze2DHeuristicManhattan;

#[inline(always)]
fn manhattan_distance(a: &Maze2DState, b: &Maze2DState) -> Maze2DCost {
    let [min_x, max_x] = std::cmp::minmax(a.x, b.x);
    let [min_y, max_y] = std::cmp::minmax(a.y, b.y);

    (max_x - min_x) + (max_y - min_y)
}

impl<P, Sp, A> Heuristic<P, Sp, Maze2DState, A, Maze2DCost> for Maze2DHeuristicManhattan
where
    P: Problem<Sp, Maze2DState, A, Maze2DCost>,
    Sp: Space<Maze2DState, A, Maze2DCost>,
    A: Action,
{
    #[inline(always)]
    fn h(p: &P, s: &Maze2DState) -> Maze2DCost {
        let mut min_c = Maze2DCost::MAX;
        for g in p.goals() {
            min_c = std::cmp::min(min_c, manhattan_distance(s, g));
        }
        min_c
    }
}
