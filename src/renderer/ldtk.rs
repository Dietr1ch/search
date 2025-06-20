// Stolen from
// `https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/helpers/ldtk.rs`

//! This example is capable of spawning tilemaps from [LDtk](https://ldtk.io) files.
//!
//! It can load the `AutoTile` and Tile layers of simple LDtk levels.
//! However, it does have limitations.
//! Some edge cases around tileset definitions and layer definitions haven't been considered here.
//! Furthermore, since this example is primarily concerned with the tilemap functionality,
//! there's no solution built in for `Entity` or `Intgrid` layers.
//!
//! For a more comprehensive LDtk solution, consider [`bevy_ecs_ldtk`](https://github.com/Trouv/bevy_ecs_ldtk), which uses `bevy_ecs_tilemap` internally.

use bevy::asset::{AssetPath, LoadContext};
use bevy::prelude::*;
use bevy_ecs_tilemap::{
    TilemapBundle,
    anchor::TilemapAnchor,
    map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};
use rustc_hash::FxHashMap;
use thiserror::Error;

#[derive(Default)]
pub struct LDtkPlugin;

impl Plugin for LDtkPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LDtkMap>()
            .register_asset_loader(LDtkLoader)
            .add_systems(Update, process_loaded_tile_maps);
    }
}

#[derive(bevy::reflect::TypePath, Asset)]
pub struct LDtkMap {
    pub project: ldtk_rust::Project,
    pub tilesets: FxHashMap<i64, Handle<Image>>,
}

#[derive(Default, Component)]
pub struct LDtkMapConfig {
    pub selected_level: usize,
}

#[derive(Default, Component)]
pub struct LDtkMapHandle(pub Handle<LDtkMap>);

#[derive(Default, Bundle)]
pub struct LDtkMapBundle {
    pub ldtk_map: LDtkMapHandle,
    pub ldtk_map_config: LDtkMapConfig,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct LDtkLoader;

#[derive(Debug, Error)]
pub enum LDtkAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load LDTk file: {0}")]
    Io(#[from] std::io::Error),
}

impl bevy::asset::AssetLoader for LDtkLoader {
    type Asset = LDtkMap;
    type Settings = ();
    type Error = LDtkAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let project: ldtk_rust::Project = serde_json::from_slice(&bytes).map_err(|e| {
            std::io::Error::other(format!("Could not read contents of LDtk map: {e}"))
        })?;
        let dependencies: Vec<(i64, AssetPath)> = project
            .defs
            .tilesets
            .iter()
            .filter_map(|tileset| {
                tileset.rel_path.as_ref().map(|rel_path| {
                    (
                        tileset.uid,
                        load_context.path().parent().unwrap().join(rel_path).into(),
                    )
                })
            })
            .collect();

        let ldtk_map = LDtkMap {
            project,
            tilesets: dependencies
                .iter()
                .map(|dep| (dep.0, load_context.load(dep.1.clone())))
                .collect(),
        };
        Ok(ldtk_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ldtk"];
        EXTENSIONS
    }
}

pub fn process_loaded_tile_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<LDtkMap>>,
    maps: Res<Assets<LDtkMap>>,
    mut query: Query<(Entity, &LDtkMapHandle, &LDtkMapConfig)>,
    new_maps: Query<&LDtkMapHandle, Added<LDtkMapHandle>>,
) {
    let mut changed_maps = Vec::<AssetId<LDtkMap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                log::info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                log::info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are OK
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // If we have new map entities, add them to the `changed_maps` list
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.0.id());
    }

    for changed_map in changed_maps.iter() {
        for (entity, map_handle, map_config) in query.iter_mut() {
            // only deal with currently changed map
            if map_handle.0.id() != *changed_map {
                continue;
            }
            if let Some(ldtk_map) = maps.get(&map_handle.0) {
                // Despawn all existing tilemaps for this LDtkMap
                commands.entity(entity).despawn_related::<Children>();

                // Pull out tilesets and their definitions into a new hashmap
                let mut tilesets = FxHashMap::default();
                ldtk_map.project.defs.tilesets.iter().for_each(|tileset| {
                    tilesets.insert(
                        tileset.uid,
                        (
                            ldtk_map.tilesets.get(&tileset.uid).unwrap().clone(),
                            tileset,
                        ),
                    );
                });

                let default_grid_size = ldtk_map.project.default_grid_size;
                let level = &ldtk_map.project.levels[map_config.selected_level];

                let map_tile_count_x = (level.px_wid / default_grid_size) as u32;
                let map_tile_count_y = (level.px_hei / default_grid_size) as u32;

                let size = TilemapSize {
                    x: map_tile_count_x,
                    y: map_tile_count_y,
                };

                // We will create a tilemap for each layer in the following loop
                for (layer_id, layer) in level
                    .layer_instances
                    .as_ref()
                    .unwrap()
                    .iter()
                    .rev()
                    .enumerate()
                {
                    if let Some(uid) = layer.tileset_def_uid {
                        let (texture, tileset) = tilesets.get(&uid).unwrap().clone();

                        // Tileset-specific tilemap settings
                        let tile_size = TilemapTileSize {
                            x: tileset.tile_grid_size as f32,
                            y: tileset.tile_grid_size as f32,
                        };

                        // Pre-emptively create a map entity for tile creation
                        let map_entity = commands.spawn_empty().id();

                        // Create tiles for this layer from LDtk's `grid_tiles` and `auto_layer_tiles`
                        let mut storage = TileStorage::empty(size);

                        for tile in layer.grid_tiles.iter().chain(layer.auto_layer_tiles.iter()) {
                            let mut position = TilePos {
                                x: (tile.px[0] / default_grid_size) as u32,
                                y: (tile.px[1] / default_grid_size) as u32,
                            };

                            position.y = map_tile_count_y - position.y - 1;

                            let tile_entity = commands
                                .spawn(TileBundle {
                                    position,
                                    tilemap_id: TilemapId(map_entity),
                                    texture_index: TileTextureIndex(tile.t as u32),
                                    ..default()
                                })
                                .id();

                            storage.set(&position, tile_entity);
                        }

                        let grid_size = tile_size.into();
                        let map_type = TilemapType::default();

                        // Create the tilemap
                        commands.entity(map_entity).insert(TilemapBundle {
                            grid_size,
                            map_type,
                            size,
                            storage,
                            texture: TilemapTexture::Single(texture),
                            tile_size,
                            anchor: TilemapAnchor::Center,
                            transform: Transform::from_xyz(0.0, 0.0, layer_id as f32),
                            ..default()
                        });
                    }
                }
            }
        }
    }
}
