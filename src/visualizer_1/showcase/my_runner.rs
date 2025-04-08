use crate::showcase::robot::MyRobot;
use robotics_lib::{runner::Runner, world::world_generator::Generator};

pub struct MyRunner(pub Runner);

impl MyRunner {
    pub fn new(robot: MyRobot, mut my_world_generator: impl Generator) -> Self {
        match Runner::new(Box::new(robot), &mut my_world_generator) {
            Ok(runner) => MyRunner(runner),
            Err(s) => panic!("Init runner error: {:?}", s),
        }
    }
}
