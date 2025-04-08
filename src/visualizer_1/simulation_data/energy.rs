use super::*;
use crate::global::MAX_ENERGY_LEVEL;
use robotics_lib::event::events::Event as RoboticLibEvent;
/// This file manages the energy init and update the energy info of the robot to the visualizer.
use std::{cmp::min, fmt::Display};

#[derive(Resource, Debug)]
pub struct MyEnergy(pub i32);

impl MyEnergy {
    pub fn recharge(&mut self, energy: usize) {
        self.0 = min(MAX_ENERGY_LEVEL as i32, self.0 + energy as i32)
    }

    pub fn consume(&mut self, energy: usize) {
        self.0 -= energy as i32
    }
}

// Needed for the init_resource function.
impl Default for MyEnergy {
    fn default() -> Self {
        MyEnergy(MAX_ENERGY_LEVEL as i32)
    }
}

impl Display for MyEnergy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct EnergyPlugin;

impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyEnergy>().add_systems(
            Update,
            update_my_energy.run_if(in_state(SimulationState::Simulation)),
        );
    }
}

/// System used to update the energy throug the events.
fn update_my_energy(mut simulation: ResMut<SimulationData>, mut my_energy: ResMut<MyEnergy>) {
    if let Some(event) = simulation.simulation_events.front() {
        // Energy recharged.
        if let MyEvent::RobLib(RoboticLibEvent::EnergyRecharged(energy)) = event {
            let mut tot_energy_recharged = *energy;
            simulation.simulation_events.pop_front();

            // the AI will stop for many tick to recharge producing many equal event
            while let Some(MyEvent::RobLib(RoboticLibEvent::EnergyRecharged(energy))) =
                simulation.simulation_events.front()
            {
                tot_energy_recharged += *energy;
                simulation.simulation_events.pop_front();
            }
            // change the energy level to be visualized
            my_energy.recharge(tot_energy_recharged);
        }
        // Same as before only for energy consumed.
        else if let MyEvent::RobLib(RoboticLibEvent::EnergyConsumed(energy)) = event {
            let mut tot_energy_consumed = *energy;
            simulation.simulation_events.pop_front();

            while let Some(MyEvent::RobLib(RoboticLibEvent::EnergyConsumed(energy))) =
                simulation.simulation_events.front()
            {
                tot_energy_consumed += *energy;
                simulation.simulation_events.pop_front();
            }
            my_energy.consume(tot_energy_consumed);
        }
    }
}
