/// This module provide a bridge from the AI to the Visualizer. The AI during its exevution will save through the event handler the
/// necessary data to run the visualizer.
use bevy::utils::hashbrown::HashSet;
use lazy_static::lazy_static;
use robotics_lib::event::events::Event as RoboticLibEvent;
use robotics_lib::world::tile::Tile;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

/// Represents the initial data for the simulation.
#[derive(Debug, Default, Clone)]
pub struct InitialData {
    pub robot_spawn_position: (usize, usize),
}

#[derive(Debug, Clone)]
pub enum MyEvent {
    RobLib(RoboticLibEvent),
    DiscoveredTiles(HashSet<(usize, usize)>),
}
lazy_static! {
    /// This HashMap holds the tiles the robot will explore, in their initial status.
    pub static ref INITIAL_MAP: Mutex<HashMap<(usize, usize), Tile>> = Mutex::new(HashMap::new());
}

lazy_static! {
    /// This VecDeque will hold all the events from the Robotic Lib fired during the execution of the AI.
    pub static ref SIMULATION_EVENTS: Mutex<VecDeque<MyEvent>> =
        Mutex::new(VecDeque::new());
}

lazy_static! {
    pub static ref SETUP_DATA: Mutex<InitialData> = Mutex::new(InitialData::default());
}

/// Writes the given event to the SIMULATION_EVENTS for the visualizer.
pub fn push_event(event: MyEvent) {
    match event {
        MyEvent::RobLib(e) => match e {
            RoboticLibEvent::Ready
            // | RoboticLibEvent::Terminated
            | RoboticLibEvent::TimeChanged(_)
            | RoboticLibEvent::DayChanged(_) => {}
            e => SIMULATION_EVENTS.lock().unwrap().push_back(MyEvent::RobLib(e)),
        },
        e => SIMULATION_EVENTS.lock().unwrap().push_back(e),
    }
}

pub fn update_initial_map(robot_map: &[Vec<Option<Tile>>], discovered: bool) {
    let mut tiles = HashSet::new();
    let mut initial_map = INITIAL_MAP.lock().unwrap();

    for (i, row) in robot_map.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            if let Some(t) = tile {
                if !initial_map.contains_key(&(i, j)) {
                    tiles.insert((i, j));
                }
                initial_map.entry((i, j)).or_insert_with(|| t.clone());
            }
        }
    }

    if discovered {
        push_event(MyEvent::DiscoveredTiles(tiles));
    }
}

/// Save the spawn posittion and world size plus useful data for the AI mission.
pub fn save_initial_data(robot_spawn_position: (usize, usize)) {
    *SETUP_DATA.lock().unwrap() = InitialData {
        robot_spawn_position,
    };
}


/// Clear the content of the initila map.
pub fn clear_previous_data() {
    INITIAL_MAP.lock().unwrap().clear();
    SIMULATION_EVENTS.lock().unwrap().clear(); 
}