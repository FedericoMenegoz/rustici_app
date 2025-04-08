use super::*;
/// This module contains all the data regarding the robot interaction with the world that need to be visualized.
use crate::global::{TILE_SIZE, WORLD_SIZE};
use ai::data_storage::*;
use bevy::utils::HashMap;
use robotics_lib::{
    event::events::Event as RoboticLibEvent,
    world::tile::{Content as RoboticLibContent, Tile as RoboticLibTile},
};
use std::{collections::VecDeque, fmt::Display};

pub(crate) mod backpack;
pub(crate) mod energy;

pub struct SimulationDataPlugIn;

impl Plugin for SimulationDataPlugIn {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldSize>()
            .init_resource::<TotalTransactions>()
            .add_systems(OnExit(SimulationState::Loading), setup_simulation_data);
    }
}

#[derive(Resource, Debug)]
pub(crate) struct WorldSize(pub(crate) usize);

/// This can be modified in the menu setting by the user.
impl Default for WorldSize {
    fn default() -> Self {
        WorldSize(WORLD_SIZE)
    }
}
/// Struct that holds the map the robot has visited, its spawn position and all the events of its interaction with the world.
#[derive(Resource, Debug)]
pub struct SimulationData {
    pub map: Vec<Vec<Option<RoboticLibTile>>>,
    pub robot_spawn_coordinate: (f32, f32),
    pub simulation_events: VecDeque<MyEvent>,
}

impl SimulationData {
    pub fn new(
        map: &std::collections::HashMap<(usize, usize), RoboticLibTile>,
        size: usize,
        robot_initial_position: (f32, f32),
        simulation_events: VecDeque<MyEvent>,
    ) -> Self {
        // The map will be translated with the bevy coordinate system.
        let map = (0..size)
            .map(|x| {
                (0..size)
                    .map(|y| map.get(&(size - 1 - y, x)).cloned())
                    .collect()
            })
            .collect();

        SimulationData {
            map,
            robot_spawn_coordinate: robot_initial_position,
            simulation_events,
        }
    }
}

/// Total coins the robot has to deposit in order to accomplish its mission.
#[derive(Resource)]
pub struct CoinsToDeposit {
    pub total: usize,
    pub deposited: usize,
}
impl CoinsToDeposit {
    fn new(total: usize) -> Self {
        CoinsToDeposit {
            total,
            deposited: 0,
        }
    }
}

/// System that set up the simulation data at after the AI has finished the computation.
fn setup_simulation_data(mut commands: Commands, world_size: Res<WorldSize>) {
    // Recover the events.
    let mut simulation_events = SIMULATION_EVENTS.lock().unwrap().clone();

    // Push a terminated event.
    simulation_events.push_back(MyEvent::RobLib(RoboticLibEvent::Terminated));

    // Recover spawn position and convert it to Bevy coordinate system.
    let setup = SETUP_DATA.lock().unwrap().clone();
    let robot_initial_position = (
        setup.robot_spawn_position.1 as f32 * TILE_SIZE,
        (world_size.0 - 1 - setup.robot_spawn_position.0) as f32 * TILE_SIZE,
    );

    let map = SimulationData::new(
        &INITIAL_MAP.lock().unwrap(),
        world_size.0,
        robot_initial_position,
        simulation_events,
    );
    let ctd = CoinsToDeposit::new((world_size.0.pow(2) as f32 * 0.002) as usize);
    let av_res = AvailableContent(HashMap::new());

    // Insert the info into Bevy.
    commands.insert_resource(map);
    commands.insert_resource(av_res);
    commands.insert_resource(ctd);
}

/// Total transaction to print on screen when the robot has finished.
#[derive(Resource, Default)]
pub(crate) struct TotalTransactions(pub(crate) HashMap<Transaction, usize>);

/// These are the only content the AI will interact with.
#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Transaction {
    CoinEarned,
    Fish,
    Wood,
    Rock,
    Garbage,
}
/// Helper function to translate from the RoboticLib and check no unintennded content will be used.
impl Transaction {
    pub fn new(content: &RoboticLibContent) -> Self {
        match content {
            RoboticLibContent::Rock(_) => Transaction::Rock,
            RoboticLibContent::Tree(_) => Transaction::Wood,
            RoboticLibContent::Garbage(_) => Transaction::Garbage,
            RoboticLibContent::Coin(_) => Transaction::CoinEarned,
            RoboticLibContent::Fish(_) => Transaction::Fish,
            _ => panic!("Not for our AI"),
        }
    }
}

/// Struct used for debug purpose, it is not visualized.
#[derive(Resource, Debug)]
pub struct AvailableContent(pub HashMap<RoboticLibContent, usize>);

impl Display for AvailableContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = "".to_string();

        for (content, qty) in self.0.iter() {
            if *qty > 0 {
                let c = match content {
                    RoboticLibContent::Rock(_) => "Rocks",
                    RoboticLibContent::Tree(_) => "Trees",
                    RoboticLibContent::Garbage(_) => "Garbage",
                    RoboticLibContent::Fire => "Fires",
                    RoboticLibContent::Coin(_) => "Coins",
                    RoboticLibContent::Bin(_) => "Bin space",
                    RoboticLibContent::Crate(_) => "Crate space",
                    RoboticLibContent::Bank(_) => "Bank space",
                    RoboticLibContent::Water(_) => "Water on land",
                    RoboticLibContent::Market(_) => "Markets operation",
                    RoboticLibContent::Fish(_) => "Fishes",
                    RoboticLibContent::Building => "Buildings",
                    RoboticLibContent::Bush(_) => "Bushes",
                    RoboticLibContent::JollyBlock(_) => "Jolly",
                    RoboticLibContent::Scarecrow => "Scarecrow",
                    RoboticLibContent::None => continue,
                };
                output += format!("{c}: {qty}\n").as_str();
            }
        }
        if output == *"" {
            output = "NO OP".to_string();
        }
        write!(f, "{}", output)
    }
}
