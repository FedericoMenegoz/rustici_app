/// This module manage the music of the app.
use self::robot::MoveRobotTimer;
use super::*;

pub(crate) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (volume_system, pitch_system).run_if(in_state(SimulationState::Simulation)),
        )
        .add_systems(OnEnter(SimulationState::Loading), loading_sound)
        .add_systems(OnEnter(SimulationState::Simulation), setup_music)
        .add_systems(OnEnter(SimulationState::Result), kill_music);
    }
}

/// Tag used to control the music during the simulation.
#[derive(Component)]
pub(crate) struct AmbientMusic;

/// Sets up the music when the simulation starts spawning the audio in loop.
fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("audio/robot_music.ogg"),
            settings: PlaybackSettings::LOOP,
        },
        AmbientMusic,
    ));
}

/// Adjusts the volume of the music based on keyboard input.
fn volume_system(
    keyboard_input: Res<Input<KeyCode>>,
    music_box_query: Query<&AudioSink, With<AmbientMusic>>,
) {
    if let Ok(sink) = music_box_query.get_single() {
        if keyboard_input.just_pressed(KeyCode::Up) {
            if sink.volume() < 2.0 {
                sink.set_volume(sink.volume() + 0.1);
            }
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            if sink.volume() > 0. {
                sink.set_volume(sink.volume() - 0.1);
            } else {
                sink.set_volume(0.0);
            }
        }
    }
}
/// Adjusts the speed of the music based on keyboard input.
fn pitch_system(
    keyboard_input: Res<Input<KeyCode>>,
    music_box_query: Query<&AudioSink, With<AmbientMusic>>,
    robot_timer: Res<MoveRobotTimer>,
) {
    if let Ok(sink) = music_box_query.get_single() {
        let millis = robot_timer.timer.duration().as_millis();
        if keyboard_input.just_pressed(KeyCode::Right) {
            if millis > 10 {
                sink.set_speed(sink.speed() + 0.025);
            }
        } else if keyboard_input.just_pressed(KeyCode::Left) {
            if millis < 1600 {
                sink.set_speed(sink.speed() - 0.025);
            }
        }
    }
}

/// System that stop the music and play a finish sound.
fn kill_music(
    mut commands: Commands,
    music_box_query: Query<Entity, With<AmbientMusic>>,
    asset_server: Res<AssetServer>,
) {
    let id = music_box_query.single();
    commands.entity(id).despawn();
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/finish.ogg"),
        settings: PlaybackSettings::DESPAWN,
    });
}
/// System that play a sound meant to play during the loading of the simulation.
fn loading_sound(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/loading.ogg"),
        settings: PlaybackSettings::DESPAWN,
    });
}