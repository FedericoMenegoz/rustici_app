/// This module contains the implementation of the LightPlugin, which is responsible for setting up and managing lights in the visualizer.
use bevy::prelude::*;

/// The LightPlugin struct represents a Bevy plugin for managing lights.
pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hex("000000").unwrap()))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 0.75,
            })
            .add_systems(Startup, spawn_light);
    }
}

/// Spawns a point light entity in the scene. Without it nothing can be seeen!
fn spawn_light(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        ..default()
    });
}
