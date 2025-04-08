use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use charting_tools::{
    charted_coordinate::ChartedCoordinate, charted_map::ChartedMap, charting_bot::ChartingBot,
};
use ordered_float::OrderedFloat;
use rand::{seq::SliceRandom, Rng};
use robotics_lib::{
    interface::{destroy, go, put, robot_map, teleport, Direction},
    runner::Runnable,
    world::{
        tile::{Content, Tile, TileType},
        World,
    },
};
use rust_eze_tomtom::{
    path::Path,
    plain::{PlainContent, PlainTileType},
};
use swift_seller::SwiftSeller;
use who_needs_gv_street_explorer::StreetExplorer;

use super::data_storage;

pub(crate) enum ActionOk {
    Completed,
}

pub(crate) enum ActionErr {
    NotFound,
    NotEnough,
    NeedsExploring,
    NotEnoughEnergy,
    Full,
}

// Function used to reach a specific tile_type in the world, if present
pub(crate) fn reach_tile_type(
    robot: &mut impl Runnable,
    world: &mut World,
    tile: PlainTileType,
) -> Result<Path, ActionErr> {
    let path = rust_eze_tomtom::TomTom::go_to_tile(robot, world, false, Some(tile), None);
    if path.is_ok() {
        return Ok(path.unwrap());
    } else {
        return Err(ActionErr::NeedsExploring);
    }
}

// Function to try to recycle the garbage in the backpack in order to get coins
pub(crate) fn recycle(robot: &mut impl Runnable) -> Result<ActionOk, ActionErr> {
    let backpack = robot.get_backpack().get_contents();
    let garbage = backpack.get(&Content::Garbage(0)).unwrap();
    let craftable = garbage / 5;

    if craftable > 0 {
        let res = recycle_by_ifrustrati::tool::recycle(robot, craftable);
        match res {
            Ok(_) => {
                return Ok(ActionOk::Completed);
            }
            Err(_) => {
                return Err(ActionErr::NotEnough);
            }
        }
    }

    return Err(ActionErr::NotEnough);
}

// Function that looks for the market with the most trades left, reaches it and sells all the sellable items in the backpack
#[allow(unused)]
pub(crate) fn sell(
    robot: &mut impl Runnable,
    world: &mut World,
    internal_map: Rc<RefCell<ChartedMap<Content>>>,
) -> Result<(usize, (usize, usize), Content), ActionErr> {
    let map = internal_map.as_ref().borrow_mut();

    // First it gets the market with the most trades left thanks to the ChartingTools
    let best_market = map.get_most(&Content::Market(0));

    // Then it tries to reach it
    let mut path = Err(String::new());
    if best_market.is_none() {
        path = rust_eze_tomtom::TomTom::get_path_to_tile(
            robot,
            world,
            false,
            None,
            Some(PlainContent::Bank),
        );
    } else {
        path = rust_eze_tomtom::TomTom::get_path_to_coordinates(
            robot,
            world,
            false,
            (
                best_market.as_ref().unwrap().0 .0,
                best_market.as_ref().unwrap().0 .1,
            ),
        );
    }

    if path.is_err() {
        return Err(ActionErr::NeedsExploring);
    }

    let result_path = path.unwrap();

    if result_path.actions.len() == 0 {
        return Err(ActionErr::NeedsExploring);
    }

    let mut steps_taken = 0;
    while steps_taken < result_path.actions.len() - 1 {
        match &result_path.actions[steps_taken] {
            rust_eze_tomtom::path::Action::Go(dir) => {
                let res = go(robot, world, dir.clone());
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
            }
            rust_eze_tomtom::path::Action::Teleport(coordinates) => {
                let res = teleport(robot, world, coordinates.clone());
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
            }
        }
        steps_taken += 1;
    }

    // Finally, it stops before reaching it in orfer not to get on the tile of the market and to know the direction needed to interact with it
    let final_action = &result_path.actions[result_path.actions.len() - 1];
    match final_action {
        rust_eze_tomtom::path::Action::Go(dir) => {
            let mut to_be_sold: Vec<Content> = Vec::new();

            let backpack = robot.get_backpack();
            for item in backpack.get_contents().into_iter() {
                if item.1 > &0
                    && (item.0 == &Content::Tree(0)
                        || item.0 == &Content::Rock(0)
                        || item.0 == &Content::Fish(0))
                {
                    to_be_sold.push(item.0.clone());
                }
            }

            let res = SwiftSeller::swift_seller(robot, world, to_be_sold);

            match res {
                Ok(items) => {
                    let mut items_sold = 0;
                    for item in items {
                        items_sold += item.1;
                    }

                    if items_sold > 0 {
                        let robot_pos = (
                            robot.get_coordinate().get_row(),
                            robot.get_coordinate().get_col(),
                        );
                        let world_map = robot_map(world).unwrap();

                        let market_position =
                            match_coordinates(robot_pos, dir.clone(), world_map.len());
                        let market_content = world_map[market_position.0][market_position.1]
                            .as_ref()
                            .unwrap()
                            .content
                            .clone();
                        return Ok((items_sold, market_position, market_content));
                    }

                    return Err(ActionErr::NotEnough);
                }
                Err(error) => match error {
                    robotics_lib::utils::LibError::NotEnoughSpace(_) => {
                        return Err(ActionErr::Full);
                    }
                    _ => return Err(ActionErr::NotEnough),
                },
            }
        }
        rust_eze_tomtom::path::Action::Teleport(_) => return Err(ActionErr::NotFound),
    }
}

// Looks for the closest desired content and destroys it
pub(crate) fn destroy_content(
    robot: &mut impl Runnable,
    world: &mut World,
    content: Content,
    internal_map: Rc<RefCell<ChartedMap<Content>>>,
) -> Result<(ActionOk, (usize, usize)), ActionErr> {
    //* NEW */
    // We get the coordinates of all the contents corresponding to the one we are looking for
    let mut map = internal_map.as_ref().borrow_mut();
    let map_of_contents = map.get(&content);

    if map_of_contents.is_none() {
        return Err(ActionErr::NeedsExploring);
    }

    let robot_pos = (
        robot.get_coordinate().get_row(),
        robot.get_coordinate().get_col(),
    );

    let desired_content = map_of_contents.unwrap().iter().min_by_key(|val| {
        OrderedFloat(f64::sqrt(
            (i32::pow(val.0 .0 as i32 - robot_pos.0 as i32, 2)
                + i32::pow(val.0 .1 as i32 - robot_pos.1 as i32, 2)) as f64,
        ))
    });

    if desired_content.is_none() {
        return Err(ActionErr::NeedsExploring);
    }

    let destination = (desired_content.unwrap().0 .0, desired_content.unwrap().0 .1);

    // It first gets to one of the tiles adjacent to the content
    let path_adjacent = rust_eze_tomtom::TomTom::go_to_coordinates(robot, world, true, destination);
    if path_adjacent.is_err() {
        return Err(ActionErr::NeedsExploring);
    }

    // Then gets where the content exactly is, if the robot isn't ina tile directly adjacent to the content it moves there, and then destroys the content using the final direction it had to move to
    let final_step =
        rust_eze_tomtom::TomTom::get_path_to_coordinates(robot, world, false, destination);
    match final_step {
        Ok(path) => {
            if path.actions.len() != 1 {
                return Err(ActionErr::NotFound);
            }

            match &path.actions[0] {
                rust_eze_tomtom::path::Action::Go(dir) => {
                    let res = destroy(robot, world, dir.clone());

                    match res {
                        Ok(_) => {
                            let robot_pos = (
                                robot.get_coordinate().get_row(),
                                robot.get_coordinate().get_col(),
                            );
                            let world_map = robot_map(world).unwrap();

                            let content_position =
                                match_coordinates(robot_pos, dir.clone(), world_map.len());

                            let _ = map.remove(
                                &content,
                                ChartedCoordinate::new(content_position.0, content_position.1),
                            );

                            return Ok((ActionOk::Completed, content_position));
                        }
                        Err(error) => match error {
                            robotics_lib::utils::LibError::NotEnoughSpace(_) => {
                                return Err(ActionErr::Full);
                            }
                            _ => {
                                return Err(ActionErr::NeedsExploring);
                            }
                        },
                    }
                }
                rust_eze_tomtom::path::Action::Teleport(_) => {
                    return Err(ActionErr::NotFound);
                }
            }
        }
        Err(_) => {
            return Err(ActionErr::NeedsExploring);
        }
    }
}

#[allow(unused)]
pub(crate) fn deposit_in_bank(
    robot: &mut impl Runnable,
    world: &mut World,
    internal_map: Rc<RefCell<ChartedMap<Content>>>,
) -> Result<(usize, (usize, usize), Content), ActionErr> {
    let map = internal_map.as_ref().borrow_mut();
    let best_bank = map.get_most(&Content::Bank(0..0));

    let mut path = Err(String::new());
    if best_bank.is_none() {
        path = rust_eze_tomtom::TomTom::get_path_to_tile(
            robot,
            world,
            false,
            None,
            Some(PlainContent::Bank),
        );
    } else {
        path = rust_eze_tomtom::TomTom::get_path_to_coordinates(
            robot,
            world,
            false,
            (
                best_bank.as_ref().unwrap().0 .0,
                best_bank.as_ref().unwrap().0 .1,
            ),
        );
    }

    if path.is_err() {
        return Err(ActionErr::NeedsExploring);
    }

    let result_path = path.unwrap();

    if result_path.actions.len() == 0 {
        return Err(ActionErr::NeedsExploring);
    }

    let mut steps_taken = 0;
    while steps_taken < result_path.actions.len() - 1 {
        match &result_path.actions[steps_taken] {
            rust_eze_tomtom::path::Action::Go(dir) => {
                let res = go(robot, world, dir.clone());
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
            }
            rust_eze_tomtom::path::Action::Teleport(coordinates) => {
                let res = teleport(robot, world, coordinates.clone());
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
            }
        }
        steps_taken += 1;
    }

    let final_action = &result_path.actions[result_path.actions.len() - 1];
    match final_action {
        rust_eze_tomtom::path::Action::Go(dir) => {
            let backpack = robot.get_backpack().get_contents();

            let coins_to_deposit = backpack.get(&Content::Coin(0)).unwrap().clone();

            let res = put(
                robot,
                world,
                Content::Coin(0),
                coins_to_deposit,
                dir.clone(),
            );

            match res {
                Ok(_) => {
                    let robot_pos = (
                        robot.get_coordinate().get_row(),
                        robot.get_coordinate().get_col(),
                    );
                    let world_map = robot_map(world).unwrap();

                    let bank_position = match_coordinates(robot_pos, dir.clone(), world_map.len());
                    let bank_content = world_map[bank_position.0][bank_position.1]
                        .as_ref()
                        .unwrap()
                        .content
                        .clone();
                    return Ok((coins_to_deposit, bank_position, bank_content));
                }
                Err(_) => return Err(ActionErr::NotFound),
            }
        }
        rust_eze_tomtom::path::Action::Teleport(_) => return Err(ActionErr::NotFound),
    }
}

// Function that explores the nearings of the robot
#[allow(unused)]
pub(crate) fn explore_nearings(
    robot: &mut impl Runnable,
    world: &mut World,
    look_distance: usize,
) -> Result<ActionOk, ActionErr> {
    let mut distance = look_distance;

    // If there are still discoverable_tiles left, it does so using the Spotlight tool
    if world.get_discoverable() > 0 {
        let res = rust_eze_spotlight::Spotlight::illuminate(robot, world, 10);

        data_storage::update_initial_map(&robot_map(world).unwrap(), true);

        if res.is_ok() {
            return Ok(ActionOk::Completed);
        }
    }

    let charter = charting_tools::ChartingTools::tool::<ChartingBot>();

    if charter.is_err() {
        return Err(ActionErr::NotFound);
    }
    let mut charter_bot = charter.unwrap();

    // Otherwise, it uses the ChartingBot from the ChartingTools
    let initial_position = (
        robot.get_coordinate().get_row(),
        robot.get_coordinate().get_col(),
    );
    let mut dir = Direction::Up;

    for i in 0..4 {
        match i {
            1 => {
                let res = rust_eze_tomtom::TomTom::go_to_coordinates(
                    robot,
                    world,
                    false,
                    initial_position,
                );
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
                dir = Direction::Down;
            }
            2 => {
                let res = rust_eze_tomtom::TomTom::go_to_coordinates(
                    robot,
                    world,
                    false,
                    initial_position,
                );
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
                dir = Direction::Left;
            }
            3 => {
                let res = rust_eze_tomtom::TomTom::go_to_coordinates(
                    robot,
                    world,
                    false,
                    initial_position,
                );
                if res.is_err() {
                    return Err(ActionErr::NotEnoughEnergy);
                }
                dir = Direction::Right;
            }
            _ => {}
        }

        // In case the robot moved, it is centered each time and then calls the charting bot in each direction
        charter_bot.init(robot);
        let res = charter_bot.discover_line(robot, world, distance, distance, dir.clone());

        if res.is_err() {
            return Err(ActionErr::NotEnoughEnergy);
        }
    }

    data_storage::update_initial_map(&robot_map(world).unwrap(), true);
    return Ok(ActionOk::Completed);
}

// Function used after having called the StreetExplorer tool. It moves the robot at the end of the street
pub(crate) fn reach_end_street(
    robot: &mut impl Runnable,
    world: &mut World,
) -> Vec<(usize, usize)> {
    let mut retval = Vec::new();

    let map = robot_map(world).unwrap();
    let size = map.len();
    let pos = (
        robot.get_coordinate().get_row(),
        robot.get_coordinate().get_col(),
    );

    let mut i = 1;

    while i < size {
        let mut starting_row = pos.0;
        if starting_row >= i {
            starting_row -= i;
        }

        let mut ending_row = pos.0;
        if ending_row < size - i {
            ending_row += i;
        }

        let mut starting_col = pos.1;
        if starting_col >= i {
            starting_col -= i;
        }

        let mut ending_col = pos.1;
        if ending_col < size - i {
            ending_col += i;
        }

        for row in starting_row..ending_row {
            if map[row][starting_col].is_none() {
                continue;
            }

            let tile = map[row][starting_col].as_ref().unwrap();
            if tile.tile_type != TileType::Street {
                continue;
            }

            match check_nearings(world, (row, starting_col)) {
                Some(destination) => retval.push(destination),
                None => {
                    continue;
                }
            }
        }

        for row in starting_row..ending_row {
            if map[row][ending_col].is_none() {
                continue;
            }

            let tile = map[row][ending_col].as_ref().unwrap();
            if tile.tile_type != TileType::Street {
                continue;
            }

            match check_nearings(world, (row, ending_col)) {
                Some(destination) => retval.push(destination),
                None => {
                    continue;
                }
            }
        }

        for col in starting_col..ending_col {
            if map[starting_row][col].is_none() {
                continue;
            }

            let tile = map[starting_row][col].as_ref().unwrap();
            if tile.tile_type != TileType::Street {
                continue;
            }

            match check_nearings(world, (starting_row, col)) {
                Some(destination) => retval.push(destination),
                None => {
                    continue;
                }
            }
        }

        for col in starting_col..ending_col {
            if map[ending_row][col].is_none() {
                continue;
            }

            let tile = map[ending_row][col].as_ref().unwrap();
            if tile.tile_type != TileType::Street {
                continue;
            }

            match check_nearings(world, (ending_row, col)) {
                Some(destination) => retval.push(destination),
                None => {
                    continue;
                }
            }
        }
        i += 1;
    }

    return retval;
}

// Functions that checks the tiles directly adjacents to a street tile, looking for the first non-ShallowWater walkable tile it can find
fn check_nearings(world: &mut World, pos: (usize, usize)) -> Option<(usize, usize)> {
    let map = robot_map(world).unwrap();
    let size = map.len();

    if pos.0 > 0 {
        let tile = &map[pos.0 - 1][pos.1];
        if tile.is_some() {
            let actual_tile = tile.as_ref().unwrap();
            match actual_tile.tile_type {
                TileType::Sand => return Some((pos.0 - 1, pos.1)),
                TileType::Grass => return Some((pos.0 - 1, pos.1)),
                TileType::Hill => return Some((pos.0 - 1, pos.1)),
                TileType::Mountain => return Some((pos.0 - 1, pos.1)),
                TileType::Snow => return Some((pos.0 - 1, pos.1)),
                _ => {}
            }
        }
    }

    if pos.0 < size - 1 {
        let tile = &map[pos.0 + 1][pos.1];
        if tile.is_some() {
            let actual_tile = tile.as_ref().unwrap();
            match actual_tile.tile_type {
                TileType::Sand => return Some((pos.0 + 1, pos.1)),
                TileType::Grass => return Some((pos.0 + 1, pos.1)),
                TileType::Hill => return Some((pos.0 + 1, pos.1)),
                TileType::Mountain => return Some((pos.0 + 1, pos.1)),
                TileType::Snow => return Some((pos.0 + 1, pos.1)),
                _ => {}
            }
        }
    }

    if pos.1 > 0 {
        let tile = &map[pos.0][pos.1 - 1];
        if tile.is_some() {
            let actual_tile = tile.as_ref().unwrap();
            match actual_tile.tile_type {
                TileType::Sand => return Some((pos.0, pos.1 - 1)),
                TileType::Grass => return Some((pos.0, pos.1 - 1)),
                TileType::Hill => return Some((pos.0, pos.1 - 1)),
                TileType::Mountain => return Some((pos.0, pos.1 - 1)),
                TileType::Snow => return Some((pos.0, pos.1 - 1)),
                _ => {}
            }
        }
    }

    if pos.1 < size - 1 {
        let tile = &map[pos.0][pos.1 + 1];
        if tile.is_some() {
            let actual_tile = tile.as_ref().unwrap();
            match actual_tile.tile_type {
                TileType::Sand => return Some((pos.0, pos.1 + 1)),
                TileType::Grass => return Some((pos.0, pos.1 + 1)),
                TileType::Hill => return Some((pos.0, pos.1 + 1)),
                TileType::Mountain => return Some((pos.0, pos.1 + 1)),
                TileType::Snow => return Some((pos.0, pos.1 + 1)),
                _ => {}
            }
        }
    }

    return None;
}

// Functions used to explore an unknown area of the map
#[allow(unused)]
pub(crate) fn explore_unknown(
    robot: &mut impl Runnable,
    world: &mut World,
) -> Result<ActionOk, ActionErr> {
    let mut rng = rand::thread_rng();

    // There is a 10% probability that it explores a street instead of reaching an unknown tile
    if rng.gen::<f64>() > 0.05 {
        let robot_pos = (
            robot.get_coordinate().get_row(),
            robot.get_coordinate().get_col(),
        );
        let map = robot_map(world).unwrap();
        let values = vec![];

        let counter = Arc::new(Mutex::new((robot_pos, map, values)));
        let mut handles = vec![];

        // Creates 4 threads, each of them looking for a random unknown tile in the map
        for _ in 0..4 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                let mut pos_map = counter.lock().unwrap();
                let val = find_unknown(&pos_map.0, &pos_map.1);
                pos_map.2.push(val);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // It checks all the results returned from the threads and chooses the nearest (considering euclidian distances), valid one
        let selected_destination = match counter
            .lock()
            .unwrap()
            .2
            .iter()
            .filter(|val| val.is_some())
            .max_by_key(|cost| {
                OrderedFloat(f64::sqrt(
                    (i32::pow(cost.as_ref().unwrap().0 as i32 - robot_pos.0 as i32, 2)
                        + i32::pow(cost.as_ref().unwrap().1 as i32 - robot_pos.1 as i32, 2))
                        as f64,
                ))
            }) {
            Some(val) => val.unwrap(),
            None => {
                return Err(ActionErr::NeedsExploring);
            }
        };

        // The robot reached the tile just before the unknown tile
        let res =
            rust_eze_tomtom::TomTom::go_to_coordinates(robot, world, false, selected_destination);

        if res.is_err() {
            return Err(ActionErr::NeedsExploring);
        }

        // Starts exploring the nearings of the unknown area
        return explore_nearings(robot, world, 5);
    }

    // It calls the StreetExplorer tool
    let res = reach_tile_type(robot, world, PlainTileType::Street);
    match res {
        Ok(_) => {
            let _ = StreetExplorer::explore_street(robot, world, None, None);

            let destinations = reach_end_street(robot, world);
            if destinations.len() == 0 {
                return Err(ActionErr::NeedsExploring);
            }

            let mut rng = rand::thread_rng();
            let destination = destinations.choose(&mut rng);
            match destination {
                Some(coordinates) => {
                    let res = rust_eze_tomtom::TomTom::go_to_coordinates(
                        robot,
                        world,
                        false,
                        coordinates.clone(),
                    );
                    match res {
                        Ok(_) => {}
                        Err(_) => return Err(ActionErr::NeedsExploring),
                    }
                    return explore_nearings(robot, world, 10);
                }
                None => return Err(ActionErr::NeedsExploring),
            }
        }
        Err(_) => return Err(ActionErr::NeedsExploring),
    }
}

// Function used in the threads spawned by the explore_unknown() function. It looks for a random unknow tile
#[allow(unused)]
pub(crate) fn find_unknown(
    robot_pos: &(usize, usize),
    map: &Vec<Vec<Option<Tile>>>,
) -> Option<(usize, usize)> {
    let mut rng = rand::thread_rng();
    let size = map.len();

    let mut pos = robot_pos.clone();
    let mut actual_tile = &map[pos.0][pos.1];

    let directions = vec![
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
    let mut dir = directions.choose(&mut rng).unwrap();
    let mut max_iterations = size * size;

    // The length of one of the sides of the world is used as an upper bound of the number of iterations to avoid infinite loops
    while max_iterations > 0 {
        dir = directions.choose(&mut rng).unwrap();
        while match_coordinates(pos, dir.clone(), size).0 > map.len()
            || match_coordinates(pos, dir.clone(), size).1 > map.len()
        {
            dir = directions.choose(&mut rng).unwrap();
        }

        actual_tile = &map[match_coordinates(pos, dir.clone(), size).0]
            [match_coordinates(pos, dir.clone(), size).1];

        if actual_tile.is_none() {
            return Some(pos);
        }

        pos = match_coordinates(pos, dir.clone(), size);
        max_iterations -= 1;
    }

    return None;
}

// Functions that given a coordinate and a direction, returns the coordinates corresponding to those the robot would reach moving to that direction
pub(crate) fn match_coordinates(
    robot_pos: (usize, usize),
    dir: Direction,
    size: usize,
) -> (usize, usize) {
    match dir {
        Direction::Up => {
            if robot_pos.0 > 0 {
                return (robot_pos.0 - 1, robot_pos.1);
            }
            return robot_pos;
        }
        Direction::Down => {
            if robot_pos.0 < size - 1 {
                return (robot_pos.0 + 1, robot_pos.1);
            }
            return robot_pos;
        }
        Direction::Left => {
            if robot_pos.1 > 0 {
                return (robot_pos.0, robot_pos.1 - 1);
            }
            return robot_pos;
        }
        Direction::Right => {
            if robot_pos.1 < size - 1 {
                return (robot_pos.0, robot_pos.1 + 1);
            }
            return robot_pos;
        }
    }
}

// Functions that matches the contents of the robotics_lib with those of the TomTom tool
// pub(crate) fn match_content(content: &Content) -> PlainContent {
//     match content {
//         Content::Rock(_) => PlainContent::Rock,
//         Content::Tree(_) => PlainContent::Tree,
//         Content::Garbage(_) => PlainContent::Garbage,
//         Content::Coin(_) => PlainContent::Coin,
//         Content::Bank(_) => PlainContent::Bank,
//         Content::Fish(_) => PlainContent::Fish,
//         _ => PlainContent::None,
//     }
// }
