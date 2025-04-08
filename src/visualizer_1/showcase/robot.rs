use ai::data_storage::{push_event, save_initial_data, update_initial_map, MyEvent};
use bevy::prelude::*;
use robotics_lib::{
    energy::Energy,
    event::events::Event as RoboticLibEvent,
    interface::{craft, destroy, go, put, robot_map, teleport, where_am_i, Direction},
    runner::{backpack::BackPack, Robot as RoboticLibRobot},
    utils::LibError as RoboticLibError,
    world::{
        coordinates::Coordinate, tile::Content as RoboticLibContent, World as RoboticLibWorld,
    },
};
use rustici_planner::tool::{self, Destination, PlannerError, PlannerResult};
/// Test AI used for debug purpose during the development of the Visualizer.
///
/// This simple AI will first explore all the map using the planner tool in
/// explore mode.
/// After it will collect resources (Rock, Tree, Fish, Garbage and Coins),
/// it will destroy them and it will sell them to a market depositing after
/// the money in a bank. It will do so using the planner tool to find the cheapest
/// path to the relative content.
///
/// This Example AI is far from perfect, it aims only to show how our tool works.
use std::collections::VecDeque;

const STATE_MESSAGE: &str = "It should always be a job and last state would kill the AI.";

#[derive(Default, Debug, Clone)]
pub enum AIState {
    #[default]
    Setup,
    Exploring,
    Recharging,
    Rock,
    Trees,
    Coin,
    Fish,
    Garbage,
    Recycle,
    Sell(RoboticLibContent),
    Deposit,
    Finish,
}

pub enum CollectResult {
    Continue,
    Deposit,
    ChangeJob,
}
#[derive(Default)]
pub struct MyRobot {
    pub robot: RoboticLibRobot,
    pub robot_spawn_position: Option<(usize, usize)>,
    pub world_size: usize,
    pub state: AIState,
    pub explored_map: f32,
    pub job_left: VecDeque<AIState>,
}

impl MyRobot {
    pub fn new() -> Self {
        let robot = RoboticLibRobot::new();

        MyRobot {
            robot,
            robot_spawn_position: None,
            ..default()
        }
    }
}

impl robotics_lib::runner::Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut RoboticLibWorld) {
        match &self.state {
            AIState::Setup => {
                self.job_left = VecDeque::from([
                    AIState::Exploring,
                    AIState::Rock,
                    AIState::Fish,
                    AIState::Trees,
                    AIState::Garbage,
                    AIState::Coin,
                    AIState::Finish,
                ]);
                let size = match robot_map(world) {
                    Some(map) => map.len(),
                    None => panic!("Could not retreive the robot map."),
                };
                let (_start_area, start_position) = where_am_i(self, world);

                self.robot_spawn_position = Some(start_position);
                self.world_size = size;
                self.state = AIState::Exploring;

                save_initial_data(start_position);
            }

            AIState::Exploring => {
                let energy = self.get_energy().get_energy_level();

                if energy < 100 {
                    self.state = AIState::Recharging;
                }
                let map = robot_map(world).expect("Already checked if map was none.");

                let res = rustici_planner::tool::Planner::planner(
                    self,
                    Destination::Unknown((energy, self.world_size)),
                    world,
                );

                if (res.is_ok() && res.as_ref().unwrap() == &PlannerResult::MapAllExplored)
                    || (res.is_err() && res.unwrap_err() == PlannerError::RestOfMapIsUnreachable)
                {
                    update_initial_map(&map, false);
                    self.job_left.pop_front();
                    self.state = self.job_left.front().unwrap().clone();
                }

                let map = robot_map(world).expect("Already checked if map was none.");
                update_initial_map(&map, false);
            }

            AIState::Recharging => {
                if self.get_energy().get_energy_level() > 900 {
                    self.state = self.job_left.front().unwrap().clone();
                }
            }

            AIState::Finish => {
                self.handle_event(RoboticLibEvent::Terminated)
            },
            AIState::Rock => {
                match collect(self, world, RoboticLibContent::Rock(0)) {
                    Ok(CollectResult::Continue) | Ok(CollectResult::Deposit) => {
                        self.state = AIState::Sell(RoboticLibContent::Rock(0))
                    }
                    Ok(CollectResult::ChangeJob) => {
                        self.job_left.pop_front();
                        self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                    }
                    Err(_e) => {
                        self.state = AIState::Recharging;
                    }
                }
            }
            AIState::Trees => {
                match collect(self, world, RoboticLibContent::Tree(0)) {
                    Ok(CollectResult::Continue) | Ok(CollectResult::Deposit) => {
                        self.state = AIState::Sell(RoboticLibContent::Tree(0))
                    }
                    Ok(CollectResult::ChangeJob) => {

                        self.job_left.pop_front();
                        self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                    }
                    Err(_e) => {

                        self.state = AIState::Recharging;
                    }
                }
            }
            AIState::Coin => {
                match collect(self, world, RoboticLibContent::Coin(0)) {
                    Ok(CollectResult::Continue) | Ok(CollectResult::Deposit) => {
                        self.state = AIState::Deposit
                    }
                    Ok(CollectResult::ChangeJob) => {
                        self.job_left.pop_front();
                        self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                    }
                    Err(_e) => {
                        self.state = AIState::Recharging;
                    }
                }
            }
            AIState::Fish => {
                match collect(self, world, RoboticLibContent::Fish(0)) {
                    Ok(CollectResult::Continue) | Ok(CollectResult::Deposit) => {
                        self.state = AIState::Sell(RoboticLibContent::Fish(0))
                    }
                    Ok(CollectResult::ChangeJob) => {
                        self.job_left.pop_front();
                        self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                    }
                    Err(_e) => {
                        self.state = AIState::Recharging;
                    }
                }
            }
            AIState::Garbage => {
                match collect(self, world, RoboticLibContent::Garbage(0)) {
                    Ok(CollectResult::Continue) | Ok(CollectResult::Deposit) => {
                        self.state = AIState::Recycle
                    }
                    Ok(CollectResult::ChangeJob) => {
                        self.job_left.pop_front();
                        self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                    }
                    Err(_e) => {
                        self.state = AIState::Recharging;
                    }
                }
            }
            AIState::Sell(content) => {
                let c = content.clone();
                match deposit(self, world, RoboticLibContent::Market(0), c.clone()) {
                    Ok(_) => {
                        self.state = AIState::Deposit;
                    }
                    Err(e) => {
                        if e == RoboticLibError::NotEnoughEnergy {
                            self.job_left.push_front(AIState::Sell(c.clone()));
                            self.state = AIState::Recharging;
                        }
                    }
                }
            }
            AIState::Deposit => {
                match deposit(
                    self,
                    world,
                    RoboticLibContent::Bank(0..0),
                    RoboticLibContent::Coin(0),
                ) {
                    Ok(_) => {
                        self.state = self.job_left.front().expect(STATE_MESSAGE).clone();

                    }
                    Err(e) => {

                        if e == RoboticLibError::NoContent {
                            self.job_left.pop_front();
                            self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                        };

                        self.job_left.push_front(AIState::Deposit);
                        self.state = AIState::Recharging;
                    }
                }
            }
            AIState::Recycle => {
                match craft(self, RoboticLibContent::Coin(0)) {
                    Ok(_) => self.state = AIState::Deposit,
                    Err(e) => {
                        print!("{e:?}");
                        if e == RoboticLibError::NotEnoughEnergy {
                            self.job_left.push_front(AIState::Recycle);
                            self.state = AIState::Recharging;
                        } else {
                            self.job_left.pop_front();
                            self.state = self.job_left.front().expect(STATE_MESSAGE).clone();
                        }
                    },
                }
            },
        }
    }

    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {
        push_event(MyEvent::RobLib(event));
    }

    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }
    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}

fn go_to_content(
    robot: &mut MyRobot,
    world: &mut RoboticLibWorld,
    actions: Vec<tool::Action>,
) -> Result<Direction, RoboticLibError> {
    for (index, action) in actions.iter().enumerate() {
        match action {
            tool::Action::Move(dir) => {
                if index != actions.len() - 1 {
                    go(robot, world, dir.clone())?;
                } else {
                    return Ok(dir.clone());
                }
            }
            tool::Action::Teleport(coord) => {
                teleport(robot, world, *coord)?; 
            }
        }
    }
    Err(RoboticLibError::OperationNotAllowed)
}

fn collect(
    robot: &mut MyRobot,
    world: &mut RoboticLibWorld,
    content: RoboticLibContent,
) -> Result<CollectResult, RoboticLibError> {
    match tool::Planner::planner(robot, Destination::Content(content.clone()), world) {
        Ok(res) => match res {
            PlannerResult::Path((path, _)) => {
                let destroy_direction = go_to_content(robot, world, path)?;
                destroy(robot, world, destroy_direction)?;
                Ok(CollectResult::Continue)
            }

            PlannerResult::RadiusExplored | PlannerResult::MapAllExplored => {
                panic!("Not supposed to get this {res:?} result!")
            }
        },

        Err(e) => match e {
            PlannerError::NoContent => {
                if let Some(content_qty) = robot.robot.backpack.get_contents().get(&content) {
                    if *content_qty > 0 {
                        return Ok(CollectResult::Deposit);
                    }
                }
                Ok(CollectResult::ChangeJob)
            }
            PlannerError::GraphGeneration
            | PlannerError::Unreachable
            | PlannerError::MaxEnergyReached
            | PlannerError::RestOfMapIsUnreachable
            | PlannerError::RoboticLibError(_) => {
                panic!("Not supposed to get {e:?} error!")
            }
        },
    }
}

fn deposit(
    robot: &mut MyRobot,
    world: &mut RoboticLibWorld,
    receiver: RoboticLibContent,
    content: RoboticLibContent,
) -> Result<(), RoboticLibError> {
    let qty_to_deposit = *robot
        .robot
        .backpack
        .get_contents()
        .get(&content)
        .unwrap_or_else(|| {
            panic!(
                "Should not be in {:?} without {:?} in backpack",
                robot.state, content
            )
        });
    if qty_to_deposit > 0 {
        match tool::Planner::planner(robot, Destination::Content(receiver.clone()), world) {
            Ok(res) => match res {
                tool::PlannerResult::Path((actions, _)) => {
                    let direction_to_put = go_to_content(robot, world, actions)?;
                    put(robot, world, content, qty_to_deposit, direction_to_put)?;
                    Ok(())
                }
                PlannerResult::RadiusExplored | PlannerResult::MapAllExplored => {
                    panic!("Not supposed to get this {res:?} result!")
                }
            },
            Err(e) => match e {
                PlannerError::NoContent => {
                    if let Some(qty_content) = robot.robot.backpack.get_contents().get(&content) {
                        if *qty_content > 0 {
                            return Err(RoboticLibError::OperationNotAllowed);
                        }
                    }
                    Err(RoboticLibError::NoContent)
                }
                PlannerError::GraphGeneration
                | PlannerError::Unreachable
                | PlannerError::MaxEnergyReached
                | PlannerError::RestOfMapIsUnreachable
                | PlannerError::RoboticLibError(_) => {
                    panic!("Not supposed to get {e:?} error!")
                }
            },
        }
    } else {
        Ok(())
    }
}
