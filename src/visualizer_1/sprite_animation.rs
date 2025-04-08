/// This module contains the component necessary for a sprite to be animated.
use super::*;

pub struct MyAnimationPlugin;

impl Plugin for MyAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_sprite.run_if(in_state(SimulationState::Simulation)),
        );
    }
}

/// Represents the indices of the first and last frames in an animation.
#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

/// Represents a timer for animation.
#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

/// Animates the sprite based on the animation indices and timer.
/// The texture atlas contains all the frame of the animation.
fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}
