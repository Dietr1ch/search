use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{AnimatedTile, TileBundle, TilePos, TileStorage, TileTextureIndex};
use rand::seq::IteratorRandom;

struct TilemapMetadata {
    path: &'static str,
    size: TilemapSize,
    tile_size: TilemapTileSize,
    grid_size: TilemapGridSize,
}

const BACKGROUND_METADATA: TilemapMetadata = TilemapMetadata {
    path: "examples/animation/tiles.png", // `://assets/examples/animation/tiles.png`
    size: TilemapSize { x: 20, y: 20 },
    tile_size: TilemapTileSize { x: 16.0, y: 16.0 },
    grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
};

const FLOWERS_METADATA: TilemapMetadata = TilemapMetadata {
    path: "examples/animation/flower_sheet.png", // `://assets/examples/animation/flower_sheet.png`
    size: TilemapSize { x: 20, y: 20 },
    tile_size: TilemapTileSize { x: 32.0, y: 32.0 },
    grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
};

pub fn create_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tilemap_entity = commands.spawn_empty().id();
    let TilemapMetadata {
        path,
        size,
        grid_size,
        tile_size,
    } = BACKGROUND_METADATA;
    let texture_handle: Handle<Image> = asset_server.load(path);
    let mut tile_storage = TileStorage::empty(size);

    fill_tilemap(
        TileTextureIndex(2), // Dark green
        size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        size,
        grid_size,
        map_type,
        tile_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });
}

pub fn create_animated_flowers(mut commands: Commands, asset_server: Res<AssetServer>) {
    let TilemapMetadata {
        path,
        size: map_size,
        grid_size,
        tile_size,
    } = FLOWERS_METADATA;
    let texture_handle: Handle<Image> = asset_server.load(path);
    let mut tile_storage = TileStorage::empty(map_size);

    let tilemap_entity = commands.spawn_empty().id();

    // Choose 10 random tiles to contain flowers.
    let mut rng = rand::rng();
    let mut indices: Vec<(u32, u32)> = Vec::with_capacity((map_size.x * map_size.y) as usize);
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            indices.push((x, y));
        }
    }
    for (x, y) in indices.into_iter().choose_multiple(&mut rng, 10) {
        let tile_pos = TilePos { x, y };
        let tile_entity = commands
            .spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                },
                // To enable animation, we must insert the `AnimatedTile` component on
                // each tile that is to be animated.
                AnimatedTile {
                    start: 0,
                    end: 13,
                    speed: 0.95,
                },
            ))
            .id();

        tile_storage.set(&tile_pos, tile_entity);
    }
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        size: map_size,
        grid_size,
        map_type,
        tile_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..Default::default()
    });
}
