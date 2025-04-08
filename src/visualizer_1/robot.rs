use super::{
    asset_loader::RobotSpriteSheet,
    simulation_data::{SimulationData, WorldSize},
    sprite_animation::{AnimationIndices, AnimationTimer},
    tile_ecs::Coordinate,
    *,
};
use crate::global::{MOVE_TIMER_SECONDS, ROBOT_Z_INDEX, TILE_SIZE};
/// This module contains the implementation of the `Robot` entity in the visualizer.
///
/// The `Robot` entity represents a robot sprite in the simulation. It is responsible for spawning the robot sprite, updating its position based on simulation events, and controlling its movement speed.
use ai::data_storage::MyEvent;
use robotics_lib::event::events::Event as RoboticLibEvent;
use std::time::Duration;

pub struct RobotSpritePlugin;

impl Plugin for RobotSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Simulation), spawn_robot_sprite)
            .insert_resource(MoveRobotTimer {
                timer: Timer::from_seconds(MOVE_TIMER_SECONDS, TimerMode::Repeating),
            })
            .add_systems(
                Update,
                (move_robot, robot_speed).run_if(in_state(SimulationState::Simulation)),
            );
    }
}

/// Timer used to regulate the Robot speed.
#[derive(Resource, Debug)]
pub struct MoveRobotTimer {
    pub(crate) timer: Timer,
}

/// Tag to easily retrieve the Robot Entity.
#[derive(Component, Debug)]
pub struct RobotTag;

/// Spawns the robot sprite in the game world.
fn spawn_robot_sprite(
    mut commands: Commands,
    sprite_sheet: Res<RobotSpriteSheet>,
    game_world: Res<SimulationData>,
) {
    let (start_x, start_y) = game_world.robot_spawn_coordinate;

    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0), // Use the image on index 0
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform::from_translation(Vec3::new(start_x, start_y, ROBOT_Z_INDEX)),
            ..default()
        },
        AnimationIndices { first: 0, last: 7 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Coordinate {
            x: (start_x / TILE_SIZE) as usize,
            y: (start_y / TILE_SIZE) as usize,
        },
        RobotTag,
    ));
}

/// Moves the robot based on simulation events.
fn move_robot(
    mut query: Query<(&mut Transform, &mut Coordinate), With<RobotTag>>,
    time: Res<Time>,
    mut move_timer: ResMut<MoveRobotTimer>,
    mut simulation_data: ResMut<SimulationData>,
    world_size: Res<WorldSize>,
) {
    move_timer.timer.tick(time.delta());
    if !move_timer.timer.just_finished() {
        return;
    }

    let (mut transform, mut coordinate) = query.single_mut();

    if let Some(MyEvent::RobLib(RoboticLibEvent::Moved(_, (x, y)))) =
        simulation_data.simulation_events.front()
    {
        transform.translation.x = *y as f32 * TILE_SIZE;
        transform.translation.y = (world_size.0 - 1 - *x) as f32 * TILE_SIZE;

        coordinate.x = *y;
        coordinate.y = world_size.0 - 1 - *x;
        simulation_data.simulation_events.pop_front();
    }
}

/// Adjusts the robot's movement speed based on keyboard input.
fn robot_speed(mut timer: ResMut<MoveRobotTimer>, keyboard: Res<Input<KeyCode>>) {
    let duration = timer.timer.duration();
    let mut millis = duration.as_millis();
    if keyboard.just_pressed(KeyCode::Right) && millis > 10 {
        millis -= 50;
    }
    if keyboard.just_pressed(KeyCode::Left) && millis < 1600 {
        millis += 50;
    }

    timer
        .timer
        .set_duration(Duration::from_millis(millis as u64));
}
