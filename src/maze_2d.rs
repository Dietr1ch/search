use rustc_hash::FxHashSet;
use std::hash::Hash;

use nonmax::NonMaxU32;

use crate::heuristic_search::Heuristic;
use crate::space::Action;
use crate::space::Cost;
use crate::space::Problem;
use crate::space::Space;
use crate::space::State;

const MAX_ELEMENTS_DISPLAYED: usize = 20;

pub type CoordIntrinsic = u32;
pub type Coord = NonMaxU32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Maze2DState {
    pub x: Coord,
    pub y: Coord,
}
impl Maze2DState {
    pub fn new(x: CoordIntrinsic, y: CoordIntrinsic) -> Option<Maze2DState> {
        Some(Maze2DState {
            x: Coord::new(x)?,
            y: Coord::new(y)?,
        })
    }
    pub fn new_from_usize(x: usize, y: usize) -> Option<Maze2DState> {
        let x = (x < CoordIntrinsic::MAX as usize).then_some(x as CoordIntrinsic)?;
        let y = (y < CoordIntrinsic::MAX as usize).then_some(y as CoordIntrinsic)?;

        Some(Maze2DState {
            x: Coord::new(x)?,
            y: Coord::new(y)?,
        })
    }
    pub fn new_from_small_usize(x: usize, y: usize) -> Maze2DState {
        debug_assert!(x < CoordIntrinsic::MAX as usize);
        debug_assert!(y < CoordIntrinsic::MAX as usize);
        let x = x as CoordIntrinsic;
        let y = y as CoordIntrinsic;

        Maze2DState {
            x: Coord::new(x).unwrap(),
            y: Coord::new(y).unwrap(),
        }
    }
    pub fn safe_dimensions(max_x: usize, max_y: usize) -> bool {
        (max_x < CoordIntrinsic::MAX as usize) && (max_y < CoordIntrinsic::MAX as usize)
    }
}
impl State for Maze2DState {}

impl Default for Maze2DState {
    fn default() -> Self {
        Maze2DState {
            x: Coord::new(0 as CoordIntrinsic).unwrap(),
            y: Coord::new(0 as CoordIntrinsic).unwrap(),
        }
    }
}

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

pub type Maze2DCost = CoordIntrinsic;
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

    pub fn dimensions(&self) -> (usize, usize) {
        if self.map.is_empty() {
            return (0, 0);
        }
        (self.map[0].len(), self.map.len())
    }
    #[inline(always)]
    fn at(&self, state: &Maze2DState) -> Maze2DCell {
        debug_assert!(self.valid(state));
        unsafe {
            *self
                .map
                .get_unchecked(state.y.get() as usize)
                .get_unchecked(state.x.get() as usize)
        }
    }

    pub fn supports_random_state() -> bool {
        true
    }
    pub fn random_state<R: rand::Rng>(&self, r: &mut R) -> Option<Maze2DState> {
        let (max_x, max_y) = self.dimensions();

        for _tries in 0..1000 {
            let x = r.random::<CoordIntrinsic>() % (max_x as CoordIntrinsic);
            let y = r.random::<CoordIntrinsic>() % (max_y as CoordIntrinsic);
            let cell: Maze2DCell = self.map[y as usize][x as usize];

            if cell == Maze2DCell::Empty {
                return Maze2DState::new(x, y);
            }
        }

        None
    }
}

impl Space<Maze2DState, Maze2DAction, Maze2DCost> for Maze2DSpace {
    #[inline(always)]
    fn apply(&self, state: &Maze2DState, action: &Maze2DAction) -> Option<Maze2DState> {
        let x = state.x.get();
        let y = state.y.get();

        #[rustfmt::skip]
        let (x, y) = match action {
            Maze2DAction::Up        => (x,     y + 1),
            Maze2DAction::Down      => (x,     y - 1),
            Maze2DAction::Left      => (x - 1, y    ),
            Maze2DAction::Right     => (x + 1, y    ),
            Maze2DAction::LeftUp    => (x - 1, y + 1),
            Maze2DAction::RightUp   => (x + 1, y + 1),
            Maze2DAction::LeftDown  => (x - 1, y - 1),
            Maze2DAction::RightDown => (x + 1, y - 1),
        };

        Some(Maze2DState {
            x: Coord::new(x)?,
            y: Coord::new(y)?,
        })
    }

    #[inline(always)]
    fn valid(&self, state: &Maze2DState) -> bool {
        let (max_x, max_y) = self.dimensions();
        let (max_x, max_y) = (max_x as CoordIntrinsic, max_y as CoordIntrinsic);

        state.x.get() < max_x && state.y.get() < max_y
    }

    /// Gets the neighbours of a given position.
    ///
    /// NOTE: These states can only be used with the current Maze
    fn neighbours(&self, state: &Maze2DState) -> Vec<(Maze2DState, Maze2DAction)> {
        let mut v = Vec::<(Maze2DState, Maze2DAction)>::new();
        let (max_x, max_y) = self.dimensions();
        debug_assert!(max_x < CoordIntrinsic::MAX as usize);
        debug_assert!(max_y < CoordIntrinsic::MAX as usize);
        let (max_x, max_y) = (max_x as CoordIntrinsic, max_y as CoordIntrinsic);

        let prev = CoordIntrinsic::MAX;
        let same = 0 as CoordIntrinsic;
        let next = 1 as CoordIntrinsic;

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
            let new_x = state.x.get().wrapping_add(dx);
            let new_y = state.y.get().wrapping_add(dy);
            if new_x < max_x && new_y < max_y {
                let s = Maze2DState {
                    x: NonMaxU32::new(new_x).unwrap(),
                    y: NonMaxU32::new(new_y).unwrap(),
                };
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

#[derive(Clone, Debug)]
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
        debug_assert!(max_x < CoordIntrinsic::MAX as usize);
        debug_assert!(max_y < CoordIntrinsic::MAX as usize);
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
                            x: Coord::new(x as CoordIntrinsic).unwrap(),
                            y: Coord::new(y as CoordIntrinsic).unwrap(),
                        });
                        Maze2DCell::Empty
                    }
                    Maze2DProblemCell::Goal => {
                        problem.goals.insert(Maze2DState {
                            x: Coord::new(x as CoordIntrinsic).unwrap(),
                            y: Coord::new(y as CoordIntrinsic).unwrap(),
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
        debug_assert!(max_x < CoordIntrinsic::MAX as usize);
        debug_assert!(max_y < CoordIntrinsic::MAX as usize);

        let mut p = Maze2DProblem {
            space: Maze2DSpace::new_empty_with_dimensions(max_x, max_y),
            starts: vec![],
            goals: FxHashSet::<Maze2DState>::default(),
        };
        let max_x = max_x as CoordIntrinsic;
        let max_y = max_y as CoordIntrinsic;

        for y in 0..max_y {
            for x in 0..max_x {
                let px = img.get_pixel(x, y);
                p.space.map[y as usize][x as usize] = match px.0 {
                    [u8::MIN, u8::MIN, u8::MIN] => Maze2DCell::Empty,
                    [u8::MAX, u8::MAX, u8::MAX] => Maze2DCell::Wall,
                    [u8::MIN, u8::MAX, u8::MIN] => {
                        // GREEN (goal)
                        p.goals.insert(Maze2DState {
                            x: Coord::new(x).unwrap(),
                            y: Coord::new(y).unwrap(),
                        });
                        Maze2DCell::Empty
                    }
                    [u8::MIN, u8::MIN, u8::MAX] => {
                        // BLUE (start)
                        p.starts.push(Maze2DState {
                            x: Coord::new(x).unwrap(),
                            y: Coord::new(y).unwrap(),
                        });
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
        debug_assert!(Maze2DState::safe_dimensions(d.0, d.1));

        writeln!(
            f,
            "Maze2DProblem({}x{}) (s:{:?}, g:{:?}):",
            d.0, d.1, self.starts, self.goals
        )?;
        let map = &self.space.map;
        for (y, line) in map.iter().enumerate().take(MAX_ELEMENTS_DISPLAYED) {
            for (x, cell) in line.iter().enumerate().take(MAX_ELEMENTS_DISPLAYED) {
                let s = Maze2DState::new_from_small_usize(x, y);

                let is_start = self.starts.contains(&s);
                let is_goal = self.goals.contains(&s);

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
    let (min_x, max_x) = (min_x.get(), max_x.get());
    let (min_y, max_y) = (min_y.get(), max_y.get());

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
