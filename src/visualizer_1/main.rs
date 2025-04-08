use asset_loader::AssetLoaderPlugin;
use bevy::{asset::io::{file::FileAssetReader, AssetSource, AssetSourceId}, prelude::*, window::WindowResolution};
use light::LightPlugin;
use music::MusicPlugin;
use robot::RobotSpritePlugin;
use simulation_data::{backpack::MyBackPackPlugin, energy::EnergyPlugin, SimulationDataPlugIn};
use sprite_animation::MyAnimationPlugin;
use tile_ecs::VisualTilePlugin;
use training_values::TrainingValuesPlugin;
use ui::{map_info_commands::UIPlugin, menu::MenuPlugin};

use self::windows_cameras::WindowsPlugin;

// Modules to store the data of the simulation and bevy systems componetn and resources.
pub(crate) mod asset_loader;
pub(crate) mod global;
pub(crate) mod light;
pub(crate) mod music;
pub(crate) mod robot;
pub(crate) mod showcase;
pub(crate) mod simulation_data;
pub(crate) mod sprite_animation;
pub(crate) mod tile_ecs;
pub(crate) mod training_values;
pub(crate) mod ui;
pub(crate) mod windows_cameras;
// Global state for the visualizer
/// States that guides the cicle of the visulalizer.
/// - Menu State: starting state from here it will change either on Loading or Training State;
/// - Training State: in this state the app will save the user custom rewards and run the trainer and exit the app
/// - Loading State: will call the AI and collect the data and start the Simulation State
/// - Simulation State: this will visualize the actual robot and the map at the end will change to Result State
/// - Result State: final state, from here you can only exit the App.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum SimulationState {
    #[default]
    Menu,
    Loading,
    Simulation,
    Training,
    Result,
}

pub(crate) fn main() {
    App::new()
        // In order to change path of the assets from the root of the project to visualizer_1 dir....
        .register_asset_source(AssetSourceId::Default, AssetSource::build().with_reader(||Box::new(FileAssetReader::new("src/visualizer_1/assets"))))
        .add_plugins((
            WindowsPlugin,
            TrainingValuesPlugin,
            EnergyPlugin,
            MyBackPackPlugin,
            AssetLoaderPlugin,
            LightPlugin,
            RobotSpritePlugin,
            MyAnimationPlugin,
            VisualTilePlugin,
            SimulationDataPlugIn,
            UIPlugin,
            MenuPlugin,
            MusicPlugin,
        ))
        .run();
}
