use std::path::PathBuf;

use clap::Parser;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use search::algorithms::astar::AStarSearch;
use search::problem::BaseProblem;
use search::problem::ObjectiveProblem;
use search::problems::maze_2d::Maze2DHeuristicDiagonalDistance;
use search::problems::maze_2d::Maze2DProblem;

#[cfg(feature = "renderer")]
use bevy::prelude::*;

#[cfg(feature = "renderer")]
fn setup(mut commands: Commands) {
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

    let seed = 0u64;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color = Color::hsl(240., rng.random::<f32>() * 0.3, rng.random::<f32>() * 0.3);
            commands.spawn((
                Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.),
            ));
        }
    }
}

#[cfg(feature = "renderer")]
fn maze_renderer(mut problems: Vec<Maze2DProblem>) {
    for (i, p) in problems.iter().enumerate() {
        let search = AStarSearch::<Maze2DHeuristicDiagonalDistance, _, _, _, _, _>::new(p.clone());
        for path in search {
            println!("Problem {i}:\n{}", path);
        }
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, search::renderer::plugins::version::startup)
        .run();
}

#[cfg(not(feature = "renderer"))]
fn maze_renderer(_problems: Vec<Maze2DProblem>) {
    println!("This requires the 'renderer' feature.");
}

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
    #[arg()]
    pub problems: Vec<PathBuf>,

    #[arg(long, default_value_t = 3u64)]
    pub num_instances: u64,
    #[arg(long, default_value_t = 2u16)]
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

    let mut problems = vec![];
    for p in &args.problems {
        let mut p = Maze2DProblem::try_from(p.as_path()).unwrap();
        for instance in 0..args.num_instances {
            let mut rng = ChaCha8Rng::seed_from_u64(instance);

            if let Some(random_problem) =
                p.randomize(&mut rng, args.instance_starts, args.instance_goals)
            {
                problems.push(random_problem);
            }
        }
    }

    maze_renderer(problems);

    Ok(())
}
