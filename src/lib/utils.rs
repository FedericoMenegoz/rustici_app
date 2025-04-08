use charting_tools::{charted_coordinate::ChartedCoordinate, charted_map::ChartedMap};
use robotics_lib::{
    energy::Energy,
    interface::{robot_map, robot_view},
    runner::{backpack::BackPack, Robot, Runnable},
    world::{
        coordinates::Coordinate,
        tile::{Content, Tile},
        World,
    },
};

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, Write},
    rc::Rc,
};

use crate::{actions::ActionErr, my_events::MyEvents2};

use super::data_storage::{self, push_event, MyEvent};

use super::actions::{self};

// All the possible actions the robot can make
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Action {
    Recycle,
    Sell,
    DestroyTree,
    DestroyRock,
    DestroyGarbage,
    DestroyCoin,
    DestroyFish,
    DepositInBank,
    ExploreNearings,
    ExploreUnknown,
}

// All the possible states the robot can be in
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum States {
    Start,
    Goal,
    Destroyed,
    Sold(usize),
    PutInBank(usize),
    Recycled,
    NeedsExploring,
    BackpackFullCoins,
    BackpackFullItems,
    Neutral,
}

// Functions that loads the q_table from file
pub(crate) fn load_q_table(
    default_rewards: bool,
) -> Result<HashMap<(States, Action), f64>, String> {
    let current_dir = std::env::current_dir().unwrap();

    let mut path = String::new();
    if default_rewards {
        path += "/q_table.txt";
    } else {
        path += "/custom_q_table.txt";
    }

    let file_path = current_dir.display().to_string() + path.as_str();

    let input = File::open(file_path);

    match input.as_ref() {
        Ok(_) => {}
        Err(e) => return Err(format!("File open: {e}")),
    }

    let buffered = BufReader::new(input.unwrap());

    let mut q_table: HashMap<(States, Action), f64> = HashMap::new();

    // States and Actions
    let states = vec![
        States::Start,
        States::Goal,
        States::Destroyed,
        States::Sold(0),
        States::PutInBank(0),
        States::Recycled,
        States::NeedsExploring,
        States::BackpackFullCoins,
        States::BackpackFullItems,
        States::Neutral,
    ];

    let actions = vec![
        Action::Recycle,
        Action::Sell,
        Action::DestroyTree,
        Action::DestroyRock,
        Action::DestroyGarbage,
        Action::DestroyCoin,
        Action::DestroyFish,
        Action::DepositInBank,
        Action::ExploreNearings,
        Action::ExploreUnknown,
    ];

    let mut q_values = Vec::new();
    for line in buffered.lines() {
        match line {
            Ok(val) => q_values.push(val),
            Err(_) => {}
        }
    }

    let mut i = 0;
    for s in states {
        let mut j = 0;
        for a in actions.iter() {
            q_table.insert(
                (s, a.clone()),
                q_values[i * actions.len() + j].parse::<f64>().unwrap(),
            );
            j += 1;
        }
        i += 1;
    }

    return Ok(q_table);
}

// Functions that writes the resulting q_table to file
pub(crate) fn write_q_table(q_table: HashMap<(States, Action), f64>, default_rewards: bool) {
    let mut path = String::new();
    if default_rewards {
        path += "q_table.txt";
    } else {
        path += "custom_q_table.txt";
    }

    let mut output = File::create(path.as_str()).unwrap();

    for val in q_table.iter() {
        let _ = write!(output, "{}\n", val.1);
    }
}

// MyRobot struct
pub struct MyRobot {
    pub robot: Robot,
    pub actual_action: Rc<RefCell<Action>>,
    pub actual_state: Rc<RefCell<States>>,
    pub charted_map: Rc<RefCell<ChartedMap<Content>>>,
    pub past_events: Rc<RefCell<VecDeque<MyEvents2>>>,
    pub map: Rc<RefCell<Vec<Vec<Vec<Option<Tile>>>>>>,
}

// Implementation of the Runner trait for the MyRobot struct
impl Runnable for MyRobot {
    // process_tick() function, what the robot does
    fn process_tick(&mut self, world: &mut World) {
        let _ = robot_view(self, world);

        // If the robot is in the Start state, it just started, so it gives the spawn position to visualizer1
        if self.actual_state.as_ref().borrow().clone() == States::Start {
            data_storage::clear_previous_data();

            let robot_spawn_position = (
                self.get_coordinate().get_row(),
                self.get_coordinate().get_col(),
            );
            data_storage::save_initial_data(robot_spawn_position);

            self.past_events
                .as_ref()
                .borrow_mut()
                .push_back(MyEvents2::RobotSpawned(robot_spawn_position));
        }

        // The robot only executes an action if it has more than 500 energy units left, otherwise it commonly doesn't manage to complete a task
        if self.get_energy().get_energy_level() > 700 {
            let old_map = robot_map(world).unwrap();
            self.map.as_ref().borrow_mut().push(old_map.clone());
            data_storage::update_initial_map(&robot_map(world).unwrap(), false);

            // A match of all the possible actions. For each of them executes the snippet of code corresponding to that action
            match self.actual_action.clone().borrow().clone() {
                Action::Recycle => match actions::recycle(self) {
                    Ok(_) => {
                        self.actual_state.as_ref().replace(States::Recycled);
                    }
                    Err(_) => {
                        self.actual_state.as_ref().replace(States::NeedsExploring);
                    }
                },
                Action::Sell => match actions::sell(self, world, Rc::clone(&self.charted_map)) {
                    Ok(n) => {
                        data_storage::update_initial_map(&robot_map(world).unwrap(), false);
                        self.past_events
                            .as_ref()
                            .borrow_mut()
                            .push_back(MyEvents2::ContentInteracted(n.2, n.1));
                        self.actual_state.as_ref().replace(States::Sold(n.0));
                    }
                    Err(error) => match error {
                        actions::ActionErr::Full => {
                            self.actual_state.as_ref().replace(check_backpack(self));
                        }
                        actions::ActionErr::NotEnough => {
                            self.actual_state.as_ref().replace(check_backpack(self));
                        }
                        _ => {
                            self.actual_state.as_ref().replace(States::NeedsExploring);
                        }
                    },
                },
                Action::DestroyTree => {
                    if full(self) {
                        self.actual_state.as_ref().replace(check_backpack(self));
                    } else {
                        match actions::destroy_content(
                            self,
                            world,
                            Content::Tree(0),
                            Rc::clone(&self.charted_map),
                        ) {
                            Ok(value) => match value.0 {
                                actions::ActionOk::Completed => {
                                    data_storage::update_initial_map(
                                        &robot_map(world).unwrap(),
                                        false,
                                    );
                                    self.past_events.as_ref().borrow_mut().push_back(
                                        MyEvents2::ContentInteracted(Content::None, value.1),
                                    );
                                    self.actual_state.as_ref().replace(States::Destroyed);
                                }
                            },
                            Err(error) => match error {
                                ActionErr::Full => {
                                    self.actual_state.as_ref().replace(check_backpack(self));
                                }
                                _ => {
                                    self.actual_state.as_ref().replace(States::NeedsExploring);
                                }
                            },
                        }
                    }
                }
                Action::DestroyRock => {
                    if full(self) {
                        self.actual_state.as_ref().replace(check_backpack(self));
                    } else {
                        match actions::destroy_content(
                            self,
                            world,
                            Content::Rock(0),
                            Rc::clone(&self.charted_map),
                        ) {
                            Ok(value) => match value.0 {
                                actions::ActionOk::Completed => {
                                    data_storage::update_initial_map(
                                        &robot_map(world).unwrap(),
                                        false,
                                    );
                                    self.past_events.as_ref().borrow_mut().push_back(
                                        MyEvents2::ContentInteracted(Content::None, value.1),
                                    );
                                    self.actual_state.as_ref().replace(States::Destroyed);
                                }
                            },
                            Err(error) => match error {
                                ActionErr::Full => {
                                    self.actual_state.as_ref().replace(check_backpack(self));
                                }
                                _ => {
                                    self.actual_state.as_ref().replace(States::NeedsExploring);
                                }
                            },
                        }
                    }
                }
                Action::DestroyGarbage => {
                    if full(self) {
                        self.actual_state.as_ref().replace(check_backpack(self));
                    } else {
                        match actions::destroy_content(
                            self,
                            world,
                            Content::Garbage(0),
                            Rc::clone(&self.charted_map),
                        ) {
                            Ok(value) => match value.0 {
                                actions::ActionOk::Completed => {
                                    data_storage::update_initial_map(
                                        &robot_map(world).unwrap(),
                                        false,
                                    );
                                    self.past_events.as_ref().borrow_mut().push_back(
                                        MyEvents2::ContentInteracted(Content::None, value.1),
                                    );
                                    self.actual_state.as_ref().replace(States::Destroyed);
                                }
                            },
                            Err(error) => match error {
                                ActionErr::Full => {
                                    self.actual_state.as_ref().replace(check_backpack(self));
                                }
                                _ => {
                                    self.actual_state.as_ref().replace(States::NeedsExploring);
                                }
                            },
                        }
                    }
                }
                Action::DestroyCoin => {
                    if full(self) {
                        self.actual_state.as_ref().replace(check_backpack(self));
                    } else {
                        match actions::destroy_content(
                            self,
                            world,
                            Content::Coin(0),
                            Rc::clone(&self.charted_map),
                        ) {
                            Ok(value) => match value.0 {
                                actions::ActionOk::Completed => {
                                    data_storage::update_initial_map(
                                        &robot_map(world).unwrap(),
                                        false,
                                    );
                                    self.past_events.as_ref().borrow_mut().push_back(
                                        MyEvents2::ContentInteracted(Content::None, value.1),
                                    );
                                    self.actual_state.as_ref().replace(States::Destroyed);
                                }
                            },
                            Err(error) => match error {
                                ActionErr::Full => {
                                    self.actual_state.as_ref().replace(check_backpack(self));
                                }
                                _ => {
                                    self.actual_state.as_ref().replace(States::NeedsExploring);
                                }
                            },
                        }
                    }
                }
                Action::DestroyFish => {
                    if full(self) {
                        self.actual_state.as_ref().replace(check_backpack(self));
                    } else {
                        match actions::destroy_content(
                            self,
                            world,
                            Content::Fish(0),
                            Rc::clone(&self.charted_map),
                        ) {
                            Ok(value) => match value.0 {
                                actions::ActionOk::Completed => {
                                    data_storage::update_initial_map(
                                        &robot_map(world).unwrap(),
                                        false,
                                    );
                                    self.past_events.as_ref().borrow_mut().push_back(
                                        MyEvents2::ContentInteracted(Content::None, value.1),
                                    );
                                    self.actual_state.as_ref().replace(States::Destroyed);
                                }
                            },
                            Err(error) => match error {
                                ActionErr::Full => {
                                    self.actual_state.as_ref().replace(check_backpack(self));
                                }
                                _ => {
                                    self.actual_state.as_ref().replace(States::NeedsExploring);
                                }
                            },
                        }
                    }
                }
                Action::DepositInBank => {
                    match actions::deposit_in_bank(self, world, Rc::clone(&self.charted_map)) {
                        Ok(n) => {
                            if n.0 > 0 {
                                data_storage::update_initial_map(&robot_map(world).unwrap(), false);
                                self.past_events
                                    .as_ref()
                                    .borrow_mut()
                                    .push_back(MyEvents2::ContentInteracted(n.2, n.1));
                                self.actual_state.as_ref().replace(States::PutInBank(n.0));
                            } else {
                                self.actual_state.as_ref().replace(check_backpack(self));
                            }
                        }
                        Err(error) => match error {
                            _ => {
                                self.actual_state.as_ref().replace(States::NeedsExploring);
                            }
                        },
                    }
                }
                Action::ExploreNearings => match actions::explore_nearings(self, world, 5) {
                    Ok(_) => {
                        data_storage::update_initial_map(&robot_map(world).unwrap(), false); // Updates map of visualizer_1
                        let mut backpack_size = self.get_backpack().get_size();
                        let backpack_content = self.get_backpack().get_contents();

                        for item in backpack_content.iter() {
                            backpack_size -= item.1.clone();
                        }

                        if backpack_size == 0 {
                            self.actual_state.as_ref().replace(check_backpack(self));
                        }

                        let new_map = robot_map(world).unwrap();
                        self.past_events
                            .as_ref()
                            .borrow_mut()
                            .push_back(MyEvents2::UsedTool(new_map)); // Updates map of visualizer_2
                        self.actual_state.as_ref().replace(States::Neutral);
                    }
                    Err(_) => {
                        let new_map = robot_map(world).unwrap();
                        self.past_events
                            .as_ref()
                            .borrow_mut()
                            .push_back(MyEvents2::UsedTool(new_map)); // Updates map of visualizer_2
                        self.actual_state.as_ref().replace(States::NeedsExploring);
                    }
                },
                Action::ExploreUnknown => match actions::explore_unknown(self, world) {
                    Ok(_) => {
                        data_storage::update_initial_map(&robot_map(world).unwrap(), false); // Updates map of visualizer_1
                        let mut backpack_size = self.get_backpack().get_size();
                        let backpack_content = self.get_backpack().get_contents();

                        for item in backpack_content.iter() {
                            backpack_size -= item.1.clone();
                        }

                        if backpack_size == 0 {
                            self.actual_state.as_ref().replace(check_backpack(self));
                        }

                        let new_map = robot_map(world).unwrap();
                        self.past_events
                            .as_ref()
                            .borrow_mut()
                            .push_back(MyEvents2::UsedTool(new_map)); // Updates map of visualizer_2
                        self.actual_state.as_ref().replace(States::Neutral);
                    }
                    Err(_) => {
                        let new_map = robot_map(world).unwrap();
                        self.past_events
                            .as_ref()
                            .borrow_mut()
                            .push_back(MyEvents2::UsedTool(new_map)); // Updates map of visualizer_2
                        self.actual_state.as_ref().replace(States::NeedsExploring);
                    }
                },
            }
            update_map(world, Rc::clone(&self.charted_map));
        } else {
            self.actual_state.replace(States::Neutral);
        }
    }

    // Pushes events to the visualizer
    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {
        push_event(MyEvent::RobLib(event.clone())); // Push events to visualizer_1

        // Push event to visualizer_2
        match event {
            robotics_lib::event::events::Event::Ready => {}
            robotics_lib::event::events::Event::Terminated => {
                self.past_events
                    .as_ref()
                    .borrow_mut()
                    .push_back(MyEvents2::Event(event));
            }
            robotics_lib::event::events::Event::TimeChanged(_) => {}
            robotics_lib::event::events::Event::DayChanged(_) => {}
            robotics_lib::event::events::Event::EnergyRecharged(_) => {}
            robotics_lib::event::events::Event::EnergyConsumed(_) => {}
            robotics_lib::event::events::Event::Moved(_, _) => {
                self.past_events
                    .as_ref()
                    .borrow_mut()
                    .push_back(MyEvents2::Event(event));
            }
            robotics_lib::event::events::Event::TileContentUpdated(_, _) => {}
            robotics_lib::event::events::Event::AddedToBackpack(_, _) => {
                self.past_events
                    .as_ref()
                    .borrow_mut()
                    .push_back(MyEvents2::Event(event));
            }
            robotics_lib::event::events::Event::RemovedFromBackpack(_, _) => {
                self.past_events
                    .as_ref()
                    .borrow_mut()
                    .push_back(MyEvents2::Event(event));
            }
        }
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

// Function used at the end of each tick to update the information in the ChartingTools, in order to easily reach the best Banks and Markets
pub(crate) fn update_map(world: &mut World, internal_map: Rc<RefCell<ChartedMap<Content>>>) {
    let map = robot_map(world).unwrap();

    for i in 0..map.len() {
        for j in 0..map.len() {
            let tile = map[i][j].as_ref();
            if tile.is_some() {
                let actuale_tile = tile.unwrap();
                match &actuale_tile.content {
                    Content::Bank(range) => {
                        if range.len() > 0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Bank(0..0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Bank(range.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Bank(0..0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Bank(0..0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    Content::Market(val) => {
                        if val > &0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Market(0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Market(val.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Market(0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Market(0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    Content::Tree(val) => {
                        if val > &0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Tree(0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Tree(val.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Market(0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Tree(0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    Content::Rock(val) => {
                        if val > &0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Rock(0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Rock(val.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Market(0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Rock(0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    Content::Fish(val) => {
                        if val > &0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Fish(0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Fish(val.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Market(0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Fish(0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    Content::Coin(val) => {
                        if val > &0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Coin(0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Coin(val.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Market(0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Coin(0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    Content::Garbage(val) => {
                        if val > &0 {
                            let _ = internal_map
                                .borrow_mut()
                                .remove(&Content::Garbage(0), ChartedCoordinate::from((i, j)));

                            let my_coordinate = &ChartedCoordinate::from((i, j));
                            internal_map
                                .borrow_mut()
                                .save(&Content::Garbage(val.clone()), &my_coordinate)
                        } else {
                            if internal_map.borrow().get(&Content::Market(0)).is_some() {
                                let _ = internal_map
                                    .borrow_mut()
                                    .remove(&Content::Garbage(0), ChartedCoordinate::from((i, j)));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub(crate) fn check_backpack(robot: &impl Runnable) -> States {
    let backpack = robot.get_backpack();
    let mut coins = 0;
    let mut items = 0;

    for item in backpack.get_contents().into_iter() {
        if *item.1 > 0 {
            match item.0 {
                Content::Coin(_) => {
                    coins += item.1;
                }
                _ => {
                    items += item.1;
                }
            }
        }
    }

    if coins > items {
        return States::BackpackFullCoins;
    }

    return States::BackpackFullItems;
}

pub(crate) fn full(robot: &impl Runnable) -> bool {
    let backpack = robot.get_backpack();
    let mut backpack_size = robot.get_backpack().get_size();

    for item in backpack.get_contents().into_iter() {
        backpack_size -= item.1;
    }

    if backpack_size == 0 {
        return true;
    }

    return false;
}
