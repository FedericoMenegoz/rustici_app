/// This module contains the implementation of the Robotic Lib backpack for storing robotic content.
/// The `MyBackPack` struct represents the backpack and is a resource that can be accessed and modified by other systems.
/// It uses a `HashMap` to store the content and their quantities.
/// The `update_my_backpack_and_make_sounds` function is the system that updates the backpack based on simulation events from the Robotic Lib.
/// It also trigger a sound effect relative to the content.
use super::*;
use bevy::utils::HashMap;
use robotics_lib::{
    event::events::Event as RoboticLibEvent, world::tile::Content as RoboticLibContent,
};
use std::fmt::{Debug, Display};

#[derive(Resource, Default)]
pub struct MyBackPack(pub HashMap<RoboticLibContent, usize>);

impl MyBackPack {
    /// Adds a specified quantity of content to the backpack.
    fn add_to_backpack(&mut self, content: RoboticLibContent, qty: usize) {
        *self.0.entry(content).or_insert(0) += qty;
    }

    /// Removes a specified quantity of content from the backpack.
    fn remove_from_backpack(&mut self, content: RoboticLibContent, qty: usize) {
        if let Some(old_qty) = self.0.remove(&content) {
            let new_qty = old_qty - qty;
            if new_qty > 0 {
                self.add_to_backpack(content, new_qty);
            }
        }
    }
}

impl Debug for MyBackPack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "Empty")
        } else {
            let mut output = String::new();
            for (item, qnt) in self.0.iter() {
                if *qnt != 0 {
                    output += format!("{item}: {qnt}\n").as_str();
                }
            }
            write!(f, "{output}")
        }
    }
}

impl Display for MyBackPack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct MyBackPackPlugin;

impl Plugin for MyBackPackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyBackPack>().add_systems(
            Update,
            update_my_backpack_and_make_sounds.run_if(in_state(SimulationState::Simulation)),
        );
    }
}

/// Updates the backpack based on simulation events and play the relative sound effect.
fn update_my_backpack_and_make_sounds(
    mut commands: Commands,
    mut simulation_data: ResMut<SimulationData>,
    mut my_backpack: ResMut<MyBackPack>,
    mut total_transaction: ResMut<TotalTransactions>,
    asset_server: Res<AssetServer>,
) {
    if let Some(event) = simulation_data.simulation_events.front() {
        // Check if the robot grabbed something or received some money after selling content in market.
        if let MyEvent::RobLib(RoboticLibEvent::AddedToBackpack(content, quantity)) = event {
            my_backpack.add_to_backpack(content.clone(), *quantity);
            // Update transactions
            *total_transaction
                .0
                .entry(Transaction::new(content))
                .or_insert(0) += quantity;
            // Load sound effect
            let path = match content {
                RoboticLibContent::Coin(_) => "audio/coin.ogg",
                RoboticLibContent::Rock(_) => "audio/rock.ogg",
                RoboticLibContent::Fish(_) => "audio/fish.ogg",
                RoboticLibContent::Tree(_) => "audio/tree.ogg",
                RoboticLibContent::Garbage(_) => "audio/garbage.ogg",
                _ => panic!("Our AI doesn't touch other stuff."),
            };
            // Play sound.
            commands.spawn(AudioBundle {
                source: asset_server.load(path),
                settings: PlaybackSettings::DESPAWN,
            });
            // Pop event in order to move on!
            simulation_data.simulation_events.pop_front();
        // Check if the robot deposited some coins in bank.
        } else if let MyEvent::RobLib(RoboticLibEvent::RemovedFromBackpack(content, quantity)) =
            event
        {
            my_backpack.remove_from_backpack(content.clone(), *quantity);

            if let RoboticLibContent::Coin(_) = content {
                // Play sound.
                commands.spawn(AudioBundle {
                    source: asset_server.load("audio/bank.ogg"),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            // Pop event in order to move on!
            simulation_data.simulation_events.pop_front();
        }
    };
}
