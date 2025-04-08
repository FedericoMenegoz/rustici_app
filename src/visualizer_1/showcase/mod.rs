/// This is a basic AI logic to show our Planner Tool and our World Generator in visualizer 1.
pub(crate) mod my_runner;
pub(crate) mod robot;
pub(crate) mod world_test;
use my_runner::MyRunner;
use robot::MyRobot;
use rustici_world_generator::{
    biomes::{biome_errors::WorldGeneratorError as RusticiWGError, BiomeType},
    World as RusticiWorld,
};
pub (crate) const WORLD_SIZE_BIOME: usize = 30;
pub (crate) const WORLD_SIZE_10X10: usize = 10;


pub fn run(world_size: usize, biome: Option<BiomeType>) {
    let my_robot = MyRobot::new();
    let mut my_runner;

    // Rustici World Generator
    if let Some(b) = biome {
        let my_world_generator =
            generate_world(world_size, b).expect("error generating the world.");
        my_runner = MyRunner::new(my_robot, my_world_generator);
    }
    // World Test 10x10
    else {
        let my_world_generator = world_test::World10X10::new();
        my_runner = MyRunner::new(my_robot, my_world_generator);
    }
    let total_tick = world_size.pow(2);
    // Run the test/debug.
    for i in 0..(total_tick) {
        match my_runner.0.game_tick() {
            Ok(_) => {
                println!("{:.2}% done!", 100 as f32 / total_tick as f32 * i as f32);
            }
            Err(e) => panic!("Error: {:?}", e),
        };
    }
}

// Helper function to build the rustici world generator.
fn generate_world(world_size: usize, biome: BiomeType) -> Result<RusticiWorld, RusticiWGError> {
    rustici_world_generator::biomes::WorldBuilder::new(world_size, biome)?.build()
}
