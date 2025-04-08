use ai::ai::REWARDS;
/// Module managing the custom reward resource for the custom training bot.
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub(crate) struct TrainingValues(pub(crate) [f64; 8]);

impl Default for TrainingValues {
    fn default() -> Self {
        TrainingValues(REWARDS)
    }
}

pub(crate) struct TrainingValuesPlugin;

impl Plugin for TrainingValuesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrainingValues>();
    }
}
