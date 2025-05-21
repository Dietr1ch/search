#[cfg(feature = "renderer")]
use std::path::PathBuf;

#[cfg(feature = "renderer")]
use clap::Parser;
#[cfg(feature = "renderer")]
use hrsw::Stopwatch;
#[cfg(feature = "renderer")]
use human_duration::human_duration;
#[cfg(feature = "renderer")]
use rand_chacha::ChaCha8Rng;
#[cfg(feature = "renderer")]
use rand_chacha::rand_core::SeedableRng;
#[cfg(feature = "renderer")]
use serde::Deserialize;
#[cfg(feature = "renderer")]
use serde::Serialize;

#[cfg(feature = "renderer")]
use search::{
    algorithms::astar::AStarSearch,
    problem::{BaseProblem, ObjectiveProblem},
    problems::maze_2d::{Maze2DCell, Maze2DHeuristicDiagonalDistance, Maze2DProblem},
    space::Space,
};

#[cfg(feature = "renderer")]
use bevy::prelude::*;

#[cfg(feature = "renderer")]
fn setup(mut commands: Commands, args: Res<Args>) {
    commands.spawn((
        Camera2d,
        bevy_pancam::PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            move_keys: bevy_pancam::DirectionKeys {
                // the keyboard buttons used to move the camera
                up: vec![KeyCode::KeyQ], // initalize the struct like this or use the provided methods for
                down: vec![KeyCode::KeyW], // common key combinations
                left: vec![KeyCode::KeyE],
                right: vec![KeyCode::KeyR],
            },
            speed: 400.,              // the speed for the keyboard movement
            enabled: true,            // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 1.,        // prevent the camera from zooming too far in
            max_scale: 40.,       // prevent the camera from zooming too far out
            min_x: f32::NEG_INFINITY, // minimum x position of the camera window
            max_x: f32::INFINITY, // maximum x position of the camera window
            min_y: f32::NEG_INFINITY, // minimum y position of the camera window
            max_y: f32::INFINITY, // maximum y position of the camera window
        },
    ));

    let mut problem = Maze2DProblem::try_from(args.problem.as_path()).unwrap();
    if problem.goals().is_empty() {
        log::info!("No problem given, attempting to generate a random one.");
        let mut rng = ChaCha8Rng::seed_from_u64(args.random_seed);
        problem = problem
            .randomize(&mut rng, args.instance_starts, args.instance_goals)
            .unwrap();
        log::info!(
            "Generated a problem with {} starts and {} goals.",
            args.instance_starts,
            args.instance_goals,
        );
    }
    let (max_x, max_y) = problem.space().dimensions();

    // Render
    const ZLEVEL_BACKGROUND: f32 = 0.0f32;
    const ZLEVEL_PROBLEM: f32 = 1.0f32;
    const ZLEVEL_PATH: f32 = 2.0f32;

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    let half_size = Some(Vec2::new(spacing / 2., spacing / 2.));

    // Colours
    let wall_colour = Color::hsl(0., 0.0, 0.2);
    let empty_colour = Color::hsl(0., 0.0, 0.8);
    // Sprites
    let start_sprite = Sprite {
        color: Color::hsl(240., 0.8, 0.8),
        custom_size,
        ..default()
    };
    let goal_sprite = Sprite {
        color: Color::hsl(120., 0.8, 0.8),
        custom_size,
        ..default()
    };
    let path_sprite = Sprite {
        color: Color::hsl(0., 0.8, 0.8),
        custom_size: half_size,
        ..default()
    };

    let last_y = max_y - 1;
    log::info!("Rendering maze...");
    for y in 0..max_y {
        for x in 0..max_x {
            let cell = problem.space().map[last_y - y][x];
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;

            let colour = match cell {
                Maze2DCell::Wall => wall_colour,
                Maze2DCell::Empty => empty_colour,
            };

            commands.spawn((
                Sprite {
                    color: colour,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, ZLEVEL_BACKGROUND),
            ));
        }
    }

    let last_y = last_y as u32;
    log::info!("Rendering starts...");
    for s in problem.starts() {
        let x = s.x.get() as f32 * spacing - offset;
        let y = (last_y - s.y.get()) as f32 * spacing - offset;
        commands.spawn((
            start_sprite.clone(),
            Transform::from_xyz(x, y, ZLEVEL_PROBLEM),
        ));
    }
    log::info!("Rendering goals...");
    for g in problem.goals() {
        let x = g.x.get() as f32 * spacing - offset;
        let y = (last_y - g.y.get()) as f32 * spacing - offset;
        commands.spawn((
            goal_sprite.clone(),
            Transform::from_xyz(x, y, ZLEVEL_PROBLEM),
        ));
    }

    // Find solution
    log::info!("Solving problem...");
    let mut search =
        AStarSearch::<Maze2DHeuristicDiagonalDistance, _, _, _, _, _>::new(problem.clone());

    let stopwatch = Stopwatch::new();
    let path = search.find_next_goal();
    let elapsed = stopwatch.elapsed();

    if let Some(path) = path {
        log::info!("Path {path}");
        log::info!("- Length: {}", path.len());
        log::info!("- Elapsed time: {}", human_duration(&elapsed));
        search.write_memory_stats(std::io::stdout().lock()).unwrap();

        if path.is_empty() {
            log::info!("Empty path, the problem is trivial");
        } else {
            log::info!("Path: (cost={})", path.cost);
            let mut s = path.start.unwrap();
            log::info!("- {}..{}", s, path.end.unwrap());

            log::info!("Rendering path...");
            let x = s.x.get() as f32 * spacing - offset;
            let y = (last_y - s.y.get()) as f32 * spacing - offset;
            commands.spawn((path_sprite.clone(), Transform::from_xyz(x, y, ZLEVEL_PATH)));

            for a in &path.actions {
                if let Some(new_state) = problem.space().apply(&s, a) {
                    s = new_state;
                    log::trace!("- {a} => {s}");

                    let x = s.x.get() as f32 * spacing - offset;
                    let y = (last_y - s.y.get()) as f32 * spacing - offset;
                    commands.spawn((path_sprite.clone(), Transform::from_xyz(x, y, ZLEVEL_PATH)));
                }
            }
            debug_assert_eq!(s, path.end.unwrap());
        }
    }

    log::info!("Done!");
}

#[cfg(feature = "renderer")]
fn maze_renderer() {
    App::new()
        .add_plugins((
            bevy_args::BevyArgsPlugin::<Args>::default(),
            DefaultPlugins,
            bevy_pancam::PanCamPlugin,
            search::renderer::plugins::VersionInfo,
        ))
        .add_systems(Startup, setup)
        .run();
}

#[cfg(not(feature = "renderer"))]
fn maze_renderer() {
    println!("This requires the 'renderer' feature.");
}

#[cfg(feature = "mem_profile")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
#[cfg(not(feature = "mem_profile"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Command line arguments
#[cfg(feature = "renderer")]
#[derive(Default, Debug, Resource, Serialize, Deserialize, Parser)]
#[clap(long_version = search::build::CLAP_LONG_VERSION)]
#[command(version, about="A simple Maze renderer", long_about = None)]
pub struct Args {
    #[arg()]
    pub problem: PathBuf,

    #[arg(long, default_value_t = 0u64)]
    pub random_seed: u64,

    #[arg(long, default_value_t = 3u64)]
    pub num_instances: u64,
    #[arg(long, default_value_t = 2u16)]
    pub instance_starts: u16,
    #[arg(long, default_value_t = 3u16)]
    pub instance_goals: u16,

    #[arg(long, default_value_t = 1usize)]
    pub num_solutions: usize,
}

fn main() -> std::io::Result<()> {
    #[cfg(feature = "coz_profile")]
    coz::thread_init();

    maze_renderer();

    Ok(())
}
