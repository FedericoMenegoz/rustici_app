/// This module provides functionality for loading and preparing assets to be rendered in the App. The default asset path in Bevy is "./assets/".
/// I defined here all the constant values relatives to the images to be rendered in the app.
///
/// The `RobotSpriteSheet` struct represents the sprite sheet for the robot asset.
/// The `TilesImages` struct represents the collection of texture atlases for different types of Robotic Lib Tiles.
/// The `ContentImages` struct represents the collection of texture atlases for different types of Robotic Lib Content.
use bevy::prelude::*;

use crate::{global::TILE_SIZE, SimulationState};

/// File to load the assets to be rendered in the App.
pub const DEFAULT_ROBOT_PATH: &str = "Robot_1.png";
pub const CUSTOM_ROBOT_PATH: &str = "Robot_2.png";

/// Const values relatives to the images present in the assets folder.
const ROCK_SIZE: (f32, f32) = (16.0, 16.0);
const TREE_SIZE: (f32, f32) = (16.0, 16.0);
const TREE_BIG_SIZE: (f32, f32) = (96.0, 96.0);
const BIN_SIZE: (f32, f32) = (32.0, 32.0);
const BUSH_SIZE: (f32, f32) = (32.0, 32.0);
const BUILDING_SIZE: (f32, f32) = (16.0, 16.0);
const FISH_SIZE: (f32, f32) = (16.0, 16.0);
const JOLLY_SIZE: (f32, f32) = (16.0, 16.0);
const SCARECROW_SIZE: (f32, f32) = (16.0, 16.0);
const GARBAGE_SIZE: (f32, f32) = (96.0, 64.0);
const FIRE_SIZE: (f32, f32) = (15.0, 20.0);
const COIN_SIZE: (f32, f32) = (32.0, 32.0);

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(SimulationState::Loading), load_robot_sprite)
            .init_resource::<DefaultRobotImages>()
            .init_resource::<CustomRobotImages>()
            .init_resource::<TilesImages>()
            .init_resource::<ContentImages>()
            .init_resource::<RobotPath>();
    }
}
#[derive(Resource)]
pub struct RobotPath(pub String);
impl Default for RobotPath {
    fn default() -> Self {
        RobotPath(DEFAULT_ROBOT_PATH.to_string())
    }
}
impl RobotPath {
    pub(crate) fn is_default(&self) -> bool {
        self.0 == DEFAULT_ROBOT_PATH
    }
}
/// Handle of the images used for the Default Robot.
#[derive(Resource)]
pub struct RobotSpriteSheet(pub Handle<TextureAtlas>);

fn load_robot_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    path: Res<RobotPath>,
) {
    let texture_handle = asset_server.load(path.0.clone());

    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 8, 1, None, None);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(RobotSpriteSheet(texture_atlas_handle));
}

#[derive(Resource)]
pub(crate) struct DefaultRobotImages(pub(crate) Vec<Handle<Image>>);

impl FromWorld for DefaultRobotImages {
    fn from_world(world: &mut World) -> Self {
        let mut images = vec![];
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        for i in 0..8 {
            images.push(asset_server.load(format!("robot_1/0{i}.png")));
        }
        DefaultRobotImages(images.into())
    }
}
#[derive(Resource)]
pub(crate) struct CustomRobotImages(pub(crate) Vec<Handle<Image>>);

impl FromWorld for CustomRobotImages {
    fn from_world(world: &mut World) -> Self {
        let mut images = vec![];
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        for i in 0..8 {
            images.push(asset_server.load(format!("robot_2/0{i}.png")));
        }
        CustomRobotImages(images.into())
    }
}
/// Helper function that return an Handle to a particular asset. I use it to prepare all the handle for the Tiles and the Content to be shown in the map.
pub fn load_prepare_atlas_handle(
    path: String,
    world: &mut World,
    columns: usize,
    size: (f32, f32),
) -> Handle<TextureAtlas> {
    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
    let texture_handle = asset_server.load(path);

    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(size.0, size.0),
        columns,
        1,
        None,
        None,
    );

    let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
    texture_atlases.add(texture_atlas)
}

/// Struct with all the handles with the Tiles media
#[derive(Resource)]
pub struct TilesImages {
    pub grass: Handle<TextureAtlas>,
    pub snow: Handle<TextureAtlas>,
    pub lava: Handle<TextureAtlas>,
    pub sand: Handle<TextureAtlas>,
    pub mountain: Handle<TextureAtlas>,
    pub hill: Handle<TextureAtlas>,
    pub shallow_water: Handle<TextureAtlas>,
    pub deep_water: Handle<TextureAtlas>,
    pub teleport: Handle<TextureAtlas>,
    pub street: Handle<TextureAtlas>,
    pub wall: Handle<TextureAtlas>,

    pub not_discovered_tile: Handle<TextureAtlas>,
}

// function to init the resource at the beginning of the simulation
impl FromWorld for TilesImages {
    fn from_world(world: &mut World) -> Self {
        TilesImages {
            grass: load_prepare_atlas_handle(
                "tiles/atlas/grass.png".to_string(),
                world,
                3,
                (TILE_SIZE, TILE_SIZE),
            ),
            snow: load_prepare_atlas_handle(
                "tiles/atlas/snow.png".to_string(),
                world,
                3,
                (TILE_SIZE, TILE_SIZE),
            ),
            lava: load_prepare_atlas_handle(
                "tiles/atlas/lava.png".to_string(),
                world,
                3,
                (TILE_SIZE, TILE_SIZE),
            ),
            sand: load_prepare_atlas_handle(
                "tiles/atlas/sand.png".to_string(),
                world,
                3,
                (TILE_SIZE, TILE_SIZE),
            ),
            mountain: load_prepare_atlas_handle(
                "tiles/mountain.png".to_string(),
                world,
                1,
                (TILE_SIZE, TILE_SIZE),
            ),
            hill: load_prepare_atlas_handle(
                "tiles/hill.png".to_string(),
                world,
                1,
                (TILE_SIZE, TILE_SIZE),
            ),
            shallow_water: load_prepare_atlas_handle(
                "tiles/atlas/shallow_water.png".to_string(),
                world,
                3,
                (TILE_SIZE, TILE_SIZE),
            ),
            deep_water: load_prepare_atlas_handle(
                "tiles/atlas/deep_water.png".to_string(),
                world,
                3,
                (TILE_SIZE, TILE_SIZE),
            ),
            teleport: load_prepare_atlas_handle(
                "tiles/teleport.png".to_string(),
                world,
                1,
                (TILE_SIZE, TILE_SIZE),
            ),
            street: load_prepare_atlas_handle(
                "tiles/atlas/street.png".to_string(),
                world,
                2,
                (TILE_SIZE, TILE_SIZE),
            ),
            wall: load_prepare_atlas_handle(
                "tiles/wall.png".to_string(),
                world,
                1,
                (TILE_SIZE, TILE_SIZE),
            ),
            not_discovered_tile: load_prepare_atlas_handle(
                "tiles/not_discovered.png".to_string(),
                world,
                1,
                (TILE_SIZE, TILE_SIZE),
            ),
        }
    }
}

/// Struct with all the handles with the content media.
#[derive(Resource, Debug)]
pub struct ContentImages {
    pub rock_grass: Handle<TextureAtlas>,
    pub rock_sand: Handle<TextureAtlas>,
    pub rock_snow: Handle<TextureAtlas>,
    pub tree_grass: Handle<TextureAtlas>,
    pub tree_mountain: Handle<TextureAtlas>,
    pub tree_snow: Handle<TextureAtlas>,
    pub tree_big: Handle<TextureAtlas>,
    pub bin: Handle<TextureAtlas>,
    pub wood_crate: Handle<TextureAtlas>,
    pub bank: Handle<TextureAtlas>,
    pub water: Handle<TextureAtlas>,
    pub market: Handle<TextureAtlas>,
    pub fish: Handle<TextureAtlas>,
    pub building: Handle<TextureAtlas>,
    pub jolly_block: Handle<TextureAtlas>,
    pub scarecrow: Handle<TextureAtlas>,

    pub bush: Handle<TextureAtlas>,
    pub garbage: Handle<TextureAtlas>,
    pub fire: Handle<TextureAtlas>,
    pub coin: Handle<TextureAtlas>,
}

impl FromWorld for ContentImages {
    /// Creates a new instance of `ContentImages` from the `World` struct.
    /// Loads and prepares various image assets using the `load_prepare_atlas_handle` function.
    /// Returns the initialized `ContentImages` struct.
    fn from_world(world: &mut World) -> Self {
        ContentImages {
            rock_grass: load_prepare_atlas_handle(
                "content/rock_grass.png".to_string(),
                world,
                3,
                ROCK_SIZE,
            ),
            rock_sand: load_prepare_atlas_handle(
                "content/rock_sand.png".to_string(),
                world,
                3,
                ROCK_SIZE,
            ),
            rock_snow: load_prepare_atlas_handle(
                "content/rock_snow.png".to_string(),
                world,
                3,
                ROCK_SIZE,
            ),
            tree_grass: load_prepare_atlas_handle(
                "content/trees.png".to_string(),
                world,
                4,
                TREE_SIZE,
            ),
            tree_mountain: load_prepare_atlas_handle(
                "content/trees_mountain.png".to_string(),
                world,
                4,
                TREE_SIZE,
            ),
            tree_snow: load_prepare_atlas_handle(
                "content/trees_snow.png".to_string(),
                world,
                4,
                TREE_SIZE,
            ),
            tree_big: load_prepare_atlas_handle(
                "content/tree_big.png".to_string(),
                world,
                14,
                TREE_BIG_SIZE,
            ),
            bin: load_prepare_atlas_handle("content/bin.png".to_string(), world, 1, BIN_SIZE),
            wood_crate: load_prepare_atlas_handle(
                "content/crate.png".to_string(),
                world,
                1,
                BUILDING_SIZE,
            ),
            bank: load_prepare_atlas_handle(
                "content/bank.png".to_string(),
                world,
                1,
                BUILDING_SIZE,
            ),
            water: load_prepare_atlas_handle(
                "content/water.png".to_string(),
                world,
                1,
                (TILE_SIZE, TILE_SIZE),
            ),
            market: load_prepare_atlas_handle(
                "content/market.png".to_string(),
                world,
                1,
                BUILDING_SIZE,
            ),
            fish: load_prepare_atlas_handle("content/fish.png".to_string(), world, 3, FISH_SIZE),
            building: load_prepare_atlas_handle(
                "content/building.png".to_string(),
                world,
                1,
                BUILDING_SIZE,
            ),
            jolly_block: load_prepare_atlas_handle(
                "content/jolly_block.png".to_string(),
                world,
                4,
                JOLLY_SIZE,
            ),
            scarecrow: load_prepare_atlas_handle(
                "content/scarecrow.png".to_string(),
                world,
                3,
                SCARECROW_SIZE,
            ),
            bush: load_prepare_atlas_handle("content/bush.png".to_string(), world, 14, BUSH_SIZE),
            garbage: load_prepare_atlas_handle(
                "content/garbage.png".to_string(),
                world,
                4,
                GARBAGE_SIZE,
            ),
            fire: load_prepare_atlas_handle("content/fire.png".to_string(), world, 4, FIRE_SIZE),
            coin: load_prepare_atlas_handle("content/coin.png".to_string(), world, 9, COIN_SIZE),
        }
    }
}
