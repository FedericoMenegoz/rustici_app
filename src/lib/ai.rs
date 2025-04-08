use std::{cell::RefCell, collections::VecDeque, env, rc::Rc};

use charting_tools::{charted_map::ChartedMap, ChartingTools};
use rand::{seq::SliceRandom, Rng};
use robotics_lib::{
    runner::{Robot, Runner},
    world::tile::{Content, Tile},
};

use crate::my_events::MyEvents2;

use super::utils::{load_q_table, write_q_table, Action, MyRobot, States};

// Hyperparameters
const ALPHA: f64 = 0.1; // LEARNING RATE
const GAMMA: f64 = 0.9; // DISCOUNT FACTOR
const EPSILON: f64 = 0.2; // EXPLOITATION VS EXPLORATION

// Default rewards
// [Start, Goal, Destroyed, Sold, PutInBank, Recycled, NeedsExploring, BackpackFull]
pub const REWARDS: [f64; 8] = [0.0, 1000.0, -0.5, -0.3, -0.1, -0.1, -1.0, -0.1];

pub fn ai(
    world_size: usize,
    rewards: Vec<f64>,
    default_rewards: bool,
) -> (
    Rc<RefCell<VecDeque<MyEvents2>>>,
    Rc<RefCell<Vec<Vec<Vec<Option<Tile>>>>>>,
) {
    env::set_var("RUST_BACKTRACE", "1");

    let mut rng = rand::thread_rng();

    if rewards.len() != 8 {
        panic!("The number of rewards inserted is not the correct one");
    }

    // Goal which ends training
    let goal = States::Goal;
    let mut coins_to_deposit = ((world_size * world_size) as f32 * 0.002) as usize; //Needed coins to reach goal
    let total_coins = coins_to_deposit;
    println!(
        "Deposited: {}%",
        ((1.0 - coins_to_deposit as f32 / total_coins as f32) * 100.0) as usize
    );

    // Initialize Q-table
    let q_table_res = load_q_table(default_rewards);
    let mut q_table = match q_table_res {
        Ok(table) => table,
        Err(e) => panic!("Q-Table loading: {e}"),
    };

    // Actions
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

    // Initialize the world generator
    let mut generator = who_needs_gv_world_generator::WorldGenerator::new(world_size);
    generator.set_biome_size(0.25);
    generator.set_cities(true);
    generator.set_teleports_and_bridges(true);
    generator.set_minimum_coin_to_deposit(coins_to_deposit * 10);
    generator.set_minimum_interaction_with_markets(coins_to_deposit * 10);
    generator.set_rivers(false);

    /***** DEBUG */

    // let mut generator = crate::debug::world_test::World10X10::new(
    // );
    /**** */

    // Initialize values that will be shared with the MyTobot struct
    let initial_state = Rc::new(RefCell::new(States::Start));
    let internal_state = Rc::new(RefCell::new(States::Start));
    let internal_action = Rc::new(RefCell::new(Action::ExploreNearings));
    let pointer_to_events = Rc::new(RefCell::new(VecDeque::new()));
    let pointer_to_content_location = Rc::new(RefCell::new(Vec::new()));

    let cm = ChartingTools::tool::<ChartedMap<Content>>();
    if cm.is_err() {
        panic!();
    }

    let internal_map = Rc::new(RefCell::new(cm.unwrap()));

    // Initialize the robot
    let my_robot = MyRobot {
        robot: Robot::new(),
        actual_action: Rc::clone(&internal_action),
        actual_state: Rc::clone(&internal_state),
        charted_map: Rc::clone(&internal_map),
        past_events: Rc::clone(&pointer_to_events),
        map: Rc::clone(&pointer_to_content_location),
    };

    let mut runner = Runner::new(Box::new(my_robot), &mut generator);

    // Until the goal isn't reached, we stay in the loop that calls the game_tick()
    while initial_state.borrow().clone() != goal {
        // Select action: exploration vs exploitation
        let action = if rng.gen::<f64>() < EPSILON {
            actions.choose(&mut rng).unwrap()
        } else {
            actions
                .iter()
                .max_by(|&&a1, &&a2| {
                    q_table[&(initial_state.as_ref().borrow().clone(), a1)]
                        .partial_cmp(&q_table[&(initial_state.as_ref().borrow().clone(), a2)])
                        .unwrap()
                })
                .unwrap()
        };

        // Execute the chosen action
        internal_action.replace(action.clone());
        let _ = runner.as_mut().unwrap().game_tick();

        // Reward function
        let reward = match &initial_state.as_ref().borrow().clone() {
            States::Start => rewards[0],
            States::Goal => rewards[1],
            States::Destroyed => rewards[2],
            States::Sold(_) => rewards[3],
            States::PutInBank(_) => rewards[4],
            States::Recycled => rewards[5],
            States::NeedsExploring => rewards[6],
            States::BackpackFullCoins => rewards[7],
            States::BackpackFullItems => rewards[7],
            States::Neutral => -1.0,
        };

        let state = internal_state.borrow().clone();

        // If the last state was PutInBank(n), we update the counter of the coind left to deposit
        match state {
            States::PutInBank(n) => {
                let mut to_subtract = n;

                while coins_to_deposit > 0 && to_subtract > 0 {
                    coins_to_deposit -= 1;
                    to_subtract -= 1;
                }

                println!(
                    "Deposited: {}%",
                    ((1.0 - coins_to_deposit as f32 / total_coins as f32) * 100.0) as usize
                );
                let _ = internal_state.replace(States::NeedsExploring);
            }
            States::Sold(_) => {
                let _ = internal_state.replace(States::NeedsExploring);
            }
            _ => {}
        }

        // Q-learning update
        let next_max = actions
            .iter()
            .map(|a| q_table[&(initial_state.as_ref().borrow().clone(), a.clone())])
            .fold(f64::MIN, f64::max);

        let q_value = q_table
            .get_mut(&(initial_state.as_ref().borrow().clone(), action.clone()))
            .unwrap();

        // q-learning function
        *q_value = *q_value * (1.0 - ALPHA) + ALPHA * (reward + GAMMA * next_max - *q_value);

        initial_state.replace(internal_state.borrow().clone());

        if coins_to_deposit <= 0 {
            initial_state.replace(States::Goal);
        }
    }

    // Write results
    write_q_table(q_table, default_rewards);

    return (pointer_to_events, pointer_to_content_location);
}
