#[cfg(feature = "renderer")]
use bevy::prelude::*;
#[cfg(feature = "renderer")]
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
#[cfg(feature = "renderer")]
use bevy_ecs_tilemap::prelude::*;
#[cfg(feature = "renderer")]
use bevy_pancam::DirectionKeys;
#[cfg(feature = "renderer")]
use bevy_pancam::PanCam;
#[cfg(feature = "renderer")]
use bevy_pancam::PanCamPlugin;

#[cfg(feature = "renderer")]
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            move_keys: DirectionKeys {
                // the keyboard buttons used to move the camera
                up: vec![KeyCode::KeyW], // initialize the struct like this or use the provided methods for
                down: vec![KeyCode::KeyS], // common key combinations
                left: vec![KeyCode::KeyA],
                right: vec![KeyCode::KeyD],
            },
            speed: 400.,              // the speed for the keyboard movement
            enabled: true,            // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.2,       // prevent the camera from zooming too far in
            max_scale: 20.,       // prevent the camera from zooming too far out
            min_x: f32::NEG_INFINITY, // minimum x position of the camera window
            max_x: f32::INFINITY, // maximum x position of the camera window
            min_y: f32::NEG_INFINITY, // minimum y position of the camera window
            max_y: f32::INFINITY, // maximum y position of the camera window
        },
    ));

    let texture_handle: Handle<Image> = asset_server.load("game_of_life_tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let mut i = 0;
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    visible: TileVisible(i % 2 == 0 || i % 7 == 0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
            i += 1;
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            anchor: TilemapAnchor::Center,
            ..Default::default()
        },
        LastUpdate(0.0),
    ));
}

#[cfg(feature = "renderer")]
#[derive(Component)]
pub struct LastUpdate(f64);

#[cfg(feature = "renderer")]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut tile_storage_query: Query<(&TileStorage, &TilemapSize, &mut LastUpdate)>,
    tile_query: Query<(Entity, &TilePos, &TileVisible)>,
) {
    let current_time = time.elapsed_secs_f64();
    let Ok((tile_storage, map_size, mut last_update)) = tile_storage_query.single_mut() else {
        return;
    };
    if current_time - last_update.0 > 0.1 {
        for (entity, position, visibility) in tile_query.iter() {
            let neighbor_count =
                Neighbors::get_square_neighboring_positions(position, map_size, true)
                    .entities(tile_storage)
                    .iter()
                    .filter(|neighbor| {
                        let (_, _, tile_visible) = tile_query.get(**neighbor).unwrap();
                        tile_visible.0
                    })
                    .count();

            let was_alive = visibility.0;

            let is_alive = match (was_alive, neighbor_count) {
                (true, x) if x < 2 => false,
                (true, 2) | (true, 3) => true,
                (true, x) if x > 3 => false,
                (false, 3) => true,
                (otherwise, _) => otherwise,
            };

            if is_alive && !was_alive {
                commands.entity(entity).insert(TileVisible(true));
            } else if !is_alive && was_alive {
                commands.entity(entity).insert(TileVisible(false));
            }
        }
        last_update.0 = current_time;
    }
}

#[cfg(feature = "renderer")]
fn game_of_life() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Game of Life Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PanCamPlugin)
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

#[cfg(not(feature = "renderer"))]
fn game_of_life() {
    println!("This requires the 'renderer' feature.");
}

fn main() {
    game_of_life();
}
