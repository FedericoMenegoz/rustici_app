use super::{
    asset_loader::{ContentImages, TilesImages},
    robot::RobotTag,
    simulation_data::{AvailableContent, CoinsToDeposit, SimulationData, WorldSize},
    sprite_animation::{AnimationIndices, AnimationTimer},
    *,
};
use crate::global::{CONTENT_Z_INDEX, TILE_SIZE, TILE_Z_INDEX};
/// This file include functionality to spawn the map at the beginning of the simulation and update it at every event happened.
/// Every tile is an entity and any optional content is an entity as well spawned as a child to the tile entity.
use ai::data_storage::MyEvent;
use bevy::utils::hashbrown::HashSet;
use rand::{rngs::ThreadRng, Rng};
use robotics_lib::{
    event::events::Event as RoboticLibEvent,
    world::tile::{Content as RoboticLibContent, Tile, TileType},
};

/// Enum used as a tag to retreive either all the content entity or the tile entity
#[derive(Component)]
enum WhatType {
    Tile(TileTag),
    Content(RoboticLibContent),
}
// Struct with the necessary info for each tile. And to fileter the queries.
#[derive(Component, Debug)]
pub struct TileTag {
    tile_type: TileType,
    content: RoboticLibContent,
}

impl TileTag {
    fn new(tile_type: TileType, content: RoboticLibContent) -> Self {
        TileTag { tile_type, content }
    }
}

/// Struct with the necessary info for each content. And to fileter the queries.
#[derive(Component)]
pub struct ContentTag(RoboticLibContent);

pub struct VisualTilePlugin;

impl Plugin for VisualTilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Simulation), spawn_tiles)
            .add_systems(
                Update,
                show_hide_map.run_if(in_state(SimulationState::Simulation)),
            )
            .add_systems(
                Update,
                (discover_tiles, update_tiles).run_if(in_state(SimulationState::Simulation)),
            );
    }
}

#[derive(Component, Debug, Hash, Clone, Eq, PartialEq)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

/// Struct used to make visible only the tiles the robot has discovered until that moment.
#[derive(Component)]
pub struct NotDiscovered(bool);
/// Coordinate using bevy coordinate system.
/// x goes from left to right and y goes from bottom to top
/// So respectively:
///     - RoboticLibMap[row][col] would be in bevy coordinate (x:col, y: size - 1 - row)
///     - (x, y) would rappresent the RoboticLibMap[size - 1 - y][x]
/// The translation appens when starting the simulation when building SimulationData Resource
impl Coordinate {
    pub fn new(x: usize, y: usize) -> Self {
        Coordinate { x, y }
    }

    /// Helper function used to make visible tiles near the robot.
    pub fn is_near(&self, other: &Self) -> bool {
        self.x.abs_diff(other.x) < 2 && self.y.abs_diff(other.y) < 2
    }
}

/// Spawn tiles entititiy with their content.
fn spawn_tiles(
    mut commands: Commands,
    tile_images: Res<TilesImages>,
    content_images: Res<ContentImages>,
    simulation_data: Res<SimulationData>,
    mut available_content: ResMut<AvailableContent>,
) {
    // Iter through the map builded in SimulationData Resource with Bevy coordinate system.
    for (i, row) in simulation_data.map.iter().enumerate() {
        for (j, tile_data) in row.iter().enumerate() {
            // Prepare thread range to pick a random sprite for an entity when they are multiple.
            let mut rng = rand::thread_rng();

            // Match all the different tiles:
            //  - texture_atlas: handle with the media loaded in the asset_loader module.
            //  - index: indicates either the first index of an animation or the actual image to pick in the atlas in case is not animated
            //  - last: last index for the animation
            //  - animation: is an animation?
            //  - scale, tile_type and content are self explanatory
            let (texture_atlas, index, last, animation, scale, tile_type, content) =
                match tile_data.as_ref() {
                    Some(tile) => match tile.tile_type {
                        TileType::DeepWater => (
                            tile_images.deep_water.clone(),
                            0,
                            2,
                            true,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::ShallowWater => (
                            tile_images.shallow_water.clone(),
                            0,
                            2,
                            true,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Sand => (
                            tile_images.sand.clone(),
                            rng.gen_range(0..3),
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Grass => (
                            tile_images.grass.clone(),
                            rng.gen_range(0..3),
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Street => (
                            tile_images.street.clone(),
                            rng.gen_range(0..2),
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Hill => (
                            tile_images.hill.clone(),
                            0,
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Mountain => (
                            tile_images.mountain.clone(),
                            0,
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Snow => (
                            tile_images.snow.clone(),
                            rng.gen_range(0..3),
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Lava => (
                            tile_images.lava.clone(),
                            0,
                            2,
                            true,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Teleport(_) => (
                            tile_images.teleport.clone(),
                            0,
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                        TileType::Wall => (
                            tile_images.wall.clone(),
                            0,
                            0,
                            false,
                            1.0,
                            tile.tile_type,
                            tile.content.clone(),
                        ),
                    },
                    None => (
                        tile_images.not_discovered_tile.clone(),
                        0,
                        0,
                        false,
                        1.0,
                        TileType::Grass,
                        RoboticLibContent::None,
                    ),
                };
            let father = spawn_entity_helper(
                &mut commands,
                texture_atlas,
                index,
                TILE_Z_INDEX,
                i,
                j,
                last,
                animation,
                scale,
                WhatType::Tile(TileTag::new(tile_type, content)),
            );

            // if animation index will be the first frame else it would be the image used in the texture atlas
            if let Some((texture_atlas, index, animation, last, scale, content)) =
                get_content_bundle(
                    tile_data.clone(),
                    &content_images,
                    rng,
                    &mut available_content,
                )
            {
                let child = spawn_entity_helper(
                    &mut commands,
                    texture_atlas,
                    index,
                    CONTENT_Z_INDEX,
                    i,
                    j,
                    last,
                    animation,
                    scale,
                    WhatType::Content(content),
                );
                commands.entity(father).add_child(child);
            }
        }
    }
}
/// Function that get info about the tile and content and prepare a bundle that will
/// hold all the components necessary to represent a content entity in Bevy.
/// It also update the value of the resource AvailableContent.
fn get_content_bundle(
    tile_data: Option<Tile>,
    content_images: &Res<ContentImages>,
    mut rng: ThreadRng,
    available_content: &mut ResMut<AvailableContent>,
) -> Option<(
    Handle<TextureAtlas>,
    usize,
    bool,
    usize,
    f32,
    RoboticLibContent,
)> {
    match tile_data.as_ref() {
        Some(tile) => match tile.content.clone() {
            RoboticLibContent::Rock(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Rock(0))
                    .or_insert(0) += q;
                Some((
                    match &tile.tile_type {
                        TileType::Sand => content_images.rock_sand.clone(),
                        TileType::Grass => content_images.rock_grass.clone(),
                        TileType::Street => content_images.rock_sand.clone(),
                        TileType::Hill => content_images.rock_sand.clone(),
                        TileType::Mountain => content_images.rock_snow.clone(),
                        TileType::Snow => content_images.rock_snow.clone(),
                        TileType::Teleport(_) => content_images.rock_sand.clone(),
                        tile_type => panic!("{tile_type:?} can't hold a Rock! Maybe?"),
                    },
                    rng.gen_range(0..3),
                    false,
                    0,
                    2.0,
                    RoboticLibContent::Rock(q),
                ))
            }
            RoboticLibContent::Tree(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Tree(0))
                    .or_insert(0) += q;
                Some((
                    content_images.tree_big.clone(),
                    0,
                    true,
                    13,
                    0.3,
                    RoboticLibContent::Tree(q),
                ))
            }
            RoboticLibContent::Garbage(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Garbage(0))
                    .or_insert(0) += q;
                Some((
                    content_images.garbage.clone(),
                    rng.gen_range(0..4),
                    false,
                    0,
                    0.3,
                    RoboticLibContent::Garbage(q),
                ))
            }
            RoboticLibContent::Fire => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Fire)
                    .or_insert(0) += 1;
                Some((
                    content_images.fire.clone(),
                    0,
                    true,
                    2,
                    1.0,
                    RoboticLibContent::Fire,
                ))
            }
            RoboticLibContent::Coin(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Coin(0))
                    .or_insert(0) += q;
                Some((
                    content_images.coin.clone(),
                    0,
                    true,
                    3,
                    0.8,
                    RoboticLibContent::Coin(q),
                ))
            }
            RoboticLibContent::Bin(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Bin(0..0))
                    .or_insert(0) += q.end;
                Some((
                    content_images.bin.clone(),
                    0,
                    false,
                    0,
                    0.5,
                    RoboticLibContent::Bin(q),
                ))
            }
            RoboticLibContent::Crate(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Crate(0..0))
                    .or_insert(0) += q.end;
                Some((
                    content_images.wood_crate.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::Crate(q),
                ))
            }
            RoboticLibContent::Bank(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Bank(0..0))
                    .or_insert(0) += q.end;
                Some((
                    content_images.bank.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::Bank(q),
                ))
            }
            RoboticLibContent::Water(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Water(0))
                    .or_insert(0) += q;
                Some((
                    content_images.water.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::Water(q),
                ))
            }
            RoboticLibContent::Market(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Market(0))
                    .or_insert(0) += q;
                Some((
                    content_images.market.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::Market(q),
                ))
            }
            RoboticLibContent::Fish(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Fish(0))
                    .or_insert(0) += q;
                Some((
                    content_images.fish.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::Fish(q),
                ))
            }
            RoboticLibContent::Building => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Building)
                    .or_insert(0) += 1;
                Some((
                    content_images.building.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::Building,
                ))
            }
            RoboticLibContent::Bush(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Bush(0))
                    .or_insert(0) += q;
                Some((
                    content_images.bush.clone(),
                    0,
                    true,
                    13,
                    1.0,
                    RoboticLibContent::Bush(q),
                ))
            }
            RoboticLibContent::JollyBlock(q) => {
                *available_content
                    .0
                    .entry(RoboticLibContent::JollyBlock(0))
                    .or_insert(0) += q;
                Some((
                    content_images.jolly_block.clone(),
                    0,
                    false,
                    0,
                    1.0,
                    RoboticLibContent::JollyBlock(q),
                ))
            }
            RoboticLibContent::Scarecrow => {
                *available_content
                    .0
                    .entry(RoboticLibContent::Scarecrow)
                    .or_insert(0) += 1;
                Some((
                    content_images.scarecrow.clone(),
                    0,
                    true,
                    2,
                    1.0,
                    RoboticLibContent::Scarecrow,
                ))
            }
            RoboticLibContent::None => None,
        },
        None => None,
    }
}

/// Helper function to spwan tiles or content and return the ID of the entity.
fn spawn_entity_helper(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
    z: f32,
    i: usize,
    j: usize,
    last: usize,
    animation: bool,
    scale: f32,
    what_type: WhatType,
) -> Entity {
    // translate the coordinate with the tile_size so that it will be placed correctly on the screen
    let (x, y) = (i as f32 * TILE_SIZE, j as f32 * TILE_SIZE);

    match what_type {
        WhatType::Tile(tile_tag) => {
            // TILE ANIMATED
            if animation {
                commands
                    .spawn((
                        SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(0),
                            transform: Transform::from_translation(Vec3::new(x, y, z))
                                .with_scale(Vec3::new(scale, scale, scale)),
                            texture_atlas,
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                        Coordinate::new(i, j),
                        NotDiscovered(false),
                        tile_tag,
                        AnimationIndices { first: 0, last },
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    ))
                    .id()
            // TILE NOT ANIMATED
            } else {
                commands
                    .spawn((
                        SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(index),
                            transform: Transform::from_translation(Vec3::new(x, y, z))
                                .with_scale(Vec3::new(scale, scale, scale)),
                            texture_atlas,
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                        Coordinate::new(i, j),
                        NotDiscovered(false),
                        tile_tag,
                    ))
                    .id()
            }
        }

        WhatType::Content(content) => {
            // CONTENT ANIMATED
            if animation {
                commands
                    .spawn((
                        SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(0),
                            transform: Transform::from_xyz(0.0, 0.0, z)
                                .with_scale(Vec3::new(scale, scale, scale)),
                            texture_atlas,
                            visibility: Visibility::Inherited,
                            ..Default::default()
                        },
                        Coordinate::new(i, j),
                        ContentTag(content),
                        AnimationIndices { first: 0, last },
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    ))
                    .id()
            // CONTENT NOT ANIMATED
            } else {
                commands
                    .spawn((
                        SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(index),
                            transform: Transform::from_xyz(0.0, 0.0, z)
                                .with_scale(Vec3::new(scale, scale, scale)),
                            texture_atlas,
                            visibility: Visibility::Inherited,
                            ..Default::default()
                        },
                        Coordinate::new(i, j),
                        ContentTag(content),
                    ))
                    .id()
            }
        }
    }
}

/// This function takes user input and shows or hides the map.
/// It won't necessarily show all the map, but only the map which has been explored by the robot at the end of the simulation.
/// And it will hide only the portion of the map which has not yet been explored.
/// The Robotic-Lib would not allow to give information about not discovered tiles.
fn show_hide_map(
    mut query: Query<(&mut Visibility, &NotDiscovered), With<TileTag>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::P) {
        for (mut visibility, discovered) in query.iter_mut() {
            if !discovered.0 {
                *visibility = Visibility::Visible
            }
        }
    }
    if keyboard_input.pressed(KeyCode::O) {
        for (mut visibility, discovered) in query.iter_mut() {
            if !discovered.0 {
                *visibility = Visibility::Hidden
            }
        }
    }
}

// this will iter throug all the tiles, a better solution would be to use bevy_spatial if you have time
// https://github.com/laundmo/bevy-spatial/tree/main
/// Function that iter through all the tile wich are not been discovered yet and check if the robot is near.
/// Also check for the event MyEvent::DiscoveredTiles which happens when the robot discover tiles without moving.
fn discover_tiles(
    mut commands: Commands,
    mut tile_query: Query<
        (&mut Visibility, &Coordinate, Entity),
        (With<TileTag>, With<NotDiscovered>),
    >,
    robot_query: Query<&Coordinate, With<RobotTag>>,
    mut simulation_data: ResMut<SimulationData>,
    world_size: Res<WorldSize>,
) {
    // check if the robot explored tiles without moving and retreive the tiles
    let mut discovered_tiles = HashSet::new();
    if let Some(MyEvent::DiscoveredTiles(tiles)) = simulation_data.simulation_events.front() {
        discovered_tiles = tiles.clone();
        simulation_data.simulation_events.pop_front();
    }
    let robot_coord = robot_query.single();
    for (mut visibility, tile_coord, id) in tile_query.iter_mut() {
        if tile_coord.is_near(robot_coord)
            || discovered_tiles.remove(&(world_size.0 - 1 - tile_coord.y, tile_coord.x))
        {
            *visibility = Visibility::Visible;
            // remove the tag not discovered
            commands.entity(id).remove::<NotDiscovered>();
        }
    }
}

/// Function that updates the tiles and their content based on the events on the SimulationData.
fn update_tiles(
    mut commands: Commands,
    mut query_tile: Query<(
        &mut TileTag,
        &Coordinate,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
        Entity,
    )>,
    mut query_content: Query<(&mut ContentTag, &Coordinate, Entity), With<ContentTag>>,
    mut simulation_data: ResMut<SimulationData>,
    world_size: Res<WorldSize>,
    content_images: Res<ContentImages>,
    tile_images: Res<TilesImages>,
    mut available_content: ResMut<AvailableContent>,
    mut coins_to_deposit: ResMut<CoinsToDeposit>,
) {
    // check if the next event is related to a tile change.
    if let Some(MyEvent::RobLib(RoboticLibEvent::TileContentUpdated(updated_tile, (row, col)))) =
        simulation_data.simulation_events.front()
    {
        // iter through all the tiles spawned in the simulation
        for (mut tile_to_check, coordinate, mut handle, mut tex_at_sprite, parent_id) in
            query_tile.iter_mut()
        {
            // check if the coordinate match,
            // remember that RoboticLibMap[row][col] would be in bevy coordinate (x:col, y: size - 1 - row)
            if coordinate.x == *col && coordinate.y == world_size.0 - 1 - *row {
                // 1. check if the tile_type is changed, if yes it must be a street tile now (the only way to change the tiletype is to put rocks and build a street)
                if tile_to_check.tile_type != updated_tile.tile_type {
                    // make the tile a street tile and represent it that way
                    *handle = tile_images.street.clone();
                    tile_to_check.tile_type = TileType::Street;

                    // in case it was an animation, we need to remove these two component as street is not animated
                    commands
                        .entity(parent_id)
                        .remove::<(AnimationIndices, AnimationTimer)>();
                    // update the sprite
                    *tex_at_sprite = TextureAtlasSprite::new(0);
                }
                // 2. case where the robot is putting stuff on a tile
                else if tile_to_check.content != updated_tile.content
                    && tile_to_check.content == RoboticLibContent::None
                {
                    if let Some((texture_atlas, index, animation, last, scale, _content)) =
                        get_content_bundle(
                            Some(updated_tile.clone()),
                            &content_images,
                            rand::thread_rng(),
                            &mut available_content,
                        )
                    {
                        let child = spawn_entity_helper(
                            &mut commands,
                            texture_atlas,
                            index,
                            CONTENT_Z_INDEX,
                            coordinate.x,
                            coordinate.y,
                            last,
                            animation,
                            scale,
                            WhatType::Content(updated_tile.content.clone()),
                        );
                        commands.entity(parent_id).add_child(child);
                    }
                }
                // 3. all the other cases
                else {
                    // iter through all the content spawned in the world
                    for (mut content, coordinate_child, child_id) in query_content.iter_mut() {
                        // check if the coordinate match,
                        // remember that RoboticLibMap[row][col] would be in bevy coordinate (x:col, y: size - 1 - row)
                        if coordinate_child.x == *col
                            && coordinate_child.y == world_size.0 - 1 - *row
                        {
                            match updated_tile.content.clone() {
                                // update the quantity on contents that can "store"
                                RoboticLibContent::Bin(new_quantity) => {
                                    // compute how much was deposited and update AvailableContent resource
                                    if let RoboticLibContent::Bin(old_qty) = content.0.clone() {
                                        let deposited = new_quantity.start - old_qty.start;

                                        *available_content
                                            .0
                                            .entry(RoboticLibContent::Bin(0..0))
                                            .or_insert(0) -= deposited;
                                    }

                                    let left = new_quantity.end - new_quantity.start;

                                    // despawn the entity if it is full
                                    if left == 0 {
                                        remove_content_entity(&mut commands, parent_id, child_id);
                                        continue;
                                    }
                                    // update the content component
                                    content.0 = RoboticLibContent::Bin(new_quantity);
                                }
                                // as above (put it on a function code repetition is baaad)
                                RoboticLibContent::Crate(new_quantity) => {
                                    if let RoboticLibContent::Crate(old_qty) = content.0.clone() {
                                        let deposited = new_quantity.start - old_qty.start;

                                        *available_content
                                            .0
                                            .entry(RoboticLibContent::Crate(0..0))
                                            .or_insert(0) -= deposited;
                                    }

                                    let left = new_quantity.end - new_quantity.start;

                                    if left == 0 {
                                        remove_content_entity(&mut commands, parent_id, child_id);
                                        continue;
                                    }

                                    content.0 = RoboticLibContent::Crate(new_quantity);
                                }

                                RoboticLibContent::Bank(new_quantity) => {
                                    if let RoboticLibContent::Bank(old_qty) = content.0.clone() {
                                        let deposited_coin = new_quantity.start - old_qty.start;

                                        coins_to_deposit.deposited += deposited_coin;
                                        *available_content
                                            .0
                                            .entry(RoboticLibContent::Bank(0..0))
                                            .or_insert(0) -= deposited_coin;
                                    }
                                    let left = new_quantity.end - new_quantity.start;

                                    if left == 0 {
                                        remove_content_entity(&mut commands, parent_id, child_id);
                                        continue;
                                    }

                                    content.0 = RoboticLibContent::Bank(new_quantity.clone());
                                }

                                RoboticLibContent::Market(new_quantity) => {
                                    *available_content
                                        .0
                                        .entry(RoboticLibContent::Market(0))
                                        .or_insert(0) = new_quantity;

                                    if new_quantity == 0 {
                                        remove_content_entity(&mut commands, parent_id, child_id);
                                        continue;
                                    }

                                    content.0 = RoboticLibContent::Market(new_quantity);
                                }

                                // Case of the updated tile with no content: aftermath of a destroy!
                                RoboticLibContent::None => {
                                    let (qty, key) = match tile_to_check.content.clone() {
                                        RoboticLibContent::Rock(qty) => {
                                            (qty, RoboticLibContent::Rock(0))
                                        }
                                        RoboticLibContent::Tree(qty) => {
                                            (qty, RoboticLibContent::Tree(0))
                                        }
                                        RoboticLibContent::Garbage(qty) => {
                                            (qty, RoboticLibContent::Garbage(0))
                                        }
                                        RoboticLibContent::Fire => (1, RoboticLibContent::Fire),
                                        RoboticLibContent::Coin(qty) => {
                                            (qty, RoboticLibContent::Coin(0))
                                        }
                                        RoboticLibContent::Water(qty) => {
                                            (qty, RoboticLibContent::Water(0))
                                        }
                                        RoboticLibContent::Fish(qty) => {
                                            (qty, RoboticLibContent::Fish(0))
                                        }
                                        RoboticLibContent::None => continue,
                                        con => panic!("{con:?} cannot become None."),
                                    };
                                    // Update available_content
                                    *available_content.0.entry(key.clone()).or_insert(0) -= qty;
                                    remove_content_entity(&mut commands, parent_id, child_id);
                                }
                                // these arms are useless, as the content will be always destroyed completely
                                new_content => content.0 = new_content,
                            }
                        }
                    }
                }
            }
        }
        simulation_data.simulation_events.pop_front();
    }
}

/// Helper function that despawn a content clearing his hierarchy first.
fn remove_content_entity(commands: &mut Commands, parent_id: Entity, child_id: Entity) {
    commands.entity(parent_id).remove_children(&[child_id]);
    commands.entity(child_id).remove_parent();
    commands.entity(child_id).despawn();
}
