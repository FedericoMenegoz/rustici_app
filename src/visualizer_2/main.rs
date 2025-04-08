pub(crate) mod backpack;
pub(crate) mod camera;
pub(crate) mod directions;
pub(crate) mod sprite_manager;
pub(crate) mod time_manager;
pub(crate) mod ui;

use macroquad::prelude::*;
use robotics_lib::{
    event::events::Event,
    world::tile::{Content, Tile, TileType},
};

use std::{collections::VecDeque, process::exit};

use ai::{ai::{ai, REWARDS}, my_events::MyEvents2};
use {
    backpack::Backpack, camera::Camera, sprite_manager::SpriteManager, time_manager::TimeManager,
};

use {directions::Direction, sprite_manager::SpritesType};

/// Screen settings
fn conf() -> Conf {
    Conf {
        window_title: String::from("Visualizer"),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let size_world = 200;
    let (pointer_to_events, pointer_to_map) = ai(size_world, REWARDS.to_vec(), true);
    let mut events = pointer_to_events.borrow_mut();
    let maps = pointer_to_map.as_ref().borrow_mut().clone();

    let mut final_map = combine_maps(maps, size_world);
    let mut robot_pos = get_starting_pos(&mut events);

    let mut time_manager = TimeManager::default();
    let sprite_manager = SpriteManager::init().await;
    let mut backpack = Backpack::new().await;
    let mut my_camera = Camera::default();

    my_camera.position = vec3(robot_pos.0 as f32, robot_pos.1 as f32 - 10., 15.);

    let mut last_mouse_position: Vec2 = mouse_position().into();
    show_mouse(false);
    set_cursor_grab(true);
    let simualtion_length = events.len() as f32;
    let mut displayed_world = vec![vec![Option::None; size_world]; size_world];
    update_tiles(&mut displayed_world, robot_pos.0, robot_pos.1, &final_map);
    displayed_world[robot_pos.0][robot_pos.1] = final_map[robot_pos.0][robot_pos.1].clone();
    let mut show_menu = true;
    while !events.is_empty() {
        if time_manager.should_update() {
            update_tick(
                &mut displayed_world,
                &mut backpack,
                &mut events,
                &mut final_map,
                &mut robot_pos,
            );
        }

        my_camera.handle_direction(&mut last_mouse_position);
        my_camera.handle_position(robot_pos, time_manager.get_speed_value());

        handle_inputs(
            &mut my_camera,
            &robot_pos,
            &mut time_manager,
            &mut show_menu,
        );
        let current_speed = time_manager.get_speed();
        let events_left = (simualtion_length - events.len() as f32) / simualtion_length * 100.;
        visualize(
            &displayed_world,
            &sprite_manager,
            &backpack,
            robot_pos,
            show_menu,
            &mut my_camera,
            current_speed,
            events_left,
        )
        .await;
        time_manager.update();
        next_frame().await
    }

    my_camera.fix_final_camera(&robot_pos);

    loop {
        my_camera.handle_direction(&mut last_mouse_position);
        my_camera.handle_position(robot_pos, time_manager.get_speed_value());
        #[rustfmt::skip]
        handle_inputs(&mut my_camera, &robot_pos, &mut time_manager, &mut show_menu);
        let current_speed = time_manager.get_speed();
        let events_left = (simualtion_length - events.len() as f32) / simualtion_length * 100.;
        visualize(
            &displayed_world,
            &sprite_manager,
            &backpack,
            robot_pos,
            show_menu,
            &mut my_camera,
            current_speed,
            events_left,
        )
        .await;
        time_manager.update();
        next_frame().await
    }
}
/// Combines all maps recieved from the AI into a final map, where everything known is in
fn combine_maps(maps: Vec<Vec<Vec<Option<Tile>>>>, size_world: usize) -> Vec<Vec<Option<Tile>>> {
    let mut final_map = vec![vec![Option::None; size_world]; size_world];
    for map in maps {
        for x in 0..size_world {
            for y in 0..size_world {
                match final_map[x][y] {
                    Some(_) => {}
                    Option::None => final_map[x][y] = map[x][y].clone(),
                }
            }
        }
    }
    return final_map;
}

/// Get starting position of the robot, consuming the first event
fn get_starting_pos(events: &mut VecDeque<MyEvents2>) -> (usize, usize) {
    let event = events.pop_front();
    let robot_pos;
    match event {
        Some(e1) => {
            match e1 {
                MyEvents2::RobotSpawned((x, y)) => {
                    robot_pos = (x, y);
                }
                _ => panic!("First event should be RobotSpawned. Not: {:?}", e1),
            };
        }
        _ => panic!("No events?"),
    };
    return robot_pos;
}

/// Calls every visual function that is needed. Indepentently of the status
async fn visualize(
    world: &Vec<Vec<Option<Tile>>>,
    sprite_manager: &SpriteManager,
    backpack: &Backpack,
    robot_pos: (usize, usize),
    show_menu: bool,
    my_camera: &mut Camera,
    current_speed: usize,
    events_left: f32,
) {
    clear_background(GRAY);

    let facing_direction = my_camera.get_look_direction();

    #[rustfmt::skip]
    draw_world(world, sprite_manager, &mut my_camera.position, &facing_direction, robot_pos).await;

    write_debug(
        &mut my_camera.position,
        my_camera.follow_robot,
        &facing_direction,
        backpack,
        current_speed,
        events_left,
        show_menu,
    )
    .await;
}

/// Updates tiles near where the robot is
fn update_tiles(
    displayed_world: &mut Vec<Vec<Option<Tile>>>,
    x: usize,
    y: usize,
    world: &Vec<Vec<Option<Tile>>>,
) {
    let size_world = world.len();
    displayed_world[x][y] = world[x][y].clone();
    if x > 0 && y > 0 {
        displayed_world[x - 1][y - 1] = world[x - 1][y - 1].clone();
    }
    if x > 0 {
        displayed_world[x - 1][y] = world[x - 1][y].clone();
    }
    if x > 0 && y < size_world - 1 {
        displayed_world[x - 1][y + 1] = world[x - 1][y + 1].clone();
    }

    if y > 0 {
        displayed_world[x][y - 1] = world[x][y - 1].clone();
    }
    if y < size_world - 1 {
        displayed_world[x][y + 1] = world[x][y + 1].clone();
    }

    if x < size_world - 1 && y > 0 {
        displayed_world[x + 1][y - 1] = world[x + 1][y - 1].clone();
    }
    if x < size_world - 1 {
        displayed_world[x + 1][y] = world[x + 1][y].clone();
    }
    if x < size_world - 1 && y < size_world - 1 {
        displayed_world[x + 1][y + 1] = world[x + 1][y + 1].clone();
    }
}

/// Updates simulation forward
fn update_tick(
    displayed_world: &mut Vec<Vec<Option<Tile>>>,
    backpack: &mut Backpack,
    events: &mut VecDeque<MyEvents2>,
    final_map: &mut Vec<Vec<Option<Tile>>>,
    robot_pos: &mut (usize, usize),
) {
    let next_event = events.pop_front();
    if next_event.is_none() {
        panic!("Events finished, but the simulation kept running.");
    }

    let next_event = next_event.unwrap();
    match next_event {
        MyEvents2::Event(Event::Moved(_, (x, y))) => {
            update_tiles(displayed_world, x, y, &final_map);
            *robot_pos = (x, y);
        }
        MyEvents2::UsedTool(new_map) => {
            let size = new_map.len();
            for i in 0..size {
                for j in 0..size {
                    if displayed_world[i][j].is_none() && new_map[i][j].is_some() {
                        displayed_world[i][j] = new_map[i][j].clone();
                    }
                }
            }
        }
        MyEvents2::ContentInteracted(content, pos) => match &content {
            Content::None => {
                final_map[pos.0][pos.1].as_mut().unwrap().content = Content::None;
                displayed_world[pos.0][pos.1].as_mut().unwrap().content = Content::None;
            }
            Content::Bank(range) => {
                if range.len() == 0 {
                    final_map[pos.0][pos.1].as_mut().unwrap().content = Content::None;
                    displayed_world[pos.0][pos.1].as_mut().unwrap().content = Content::None;
                }
            }
            Content::Market(amount) => {
                if *amount == 0 {
                    final_map[pos.0][pos.1].as_mut().unwrap().content = Content::None;
                    displayed_world[pos.0][pos.1].as_mut().unwrap().content = Content::None;
                }
            }
            _ => {}
        },
        MyEvents2::RobotSpawned(_) => {}

        MyEvents2::Event(Event::AddedToBackpack(content, quantity)) => match content {
            Content::Rock(_) => backpack.add(backpack::BackpackContent::Rock, quantity),
            Content::Tree(_) => backpack.add(backpack::BackpackContent::Tree, quantity),
            Content::Coin(_) => backpack.add(backpack::BackpackContent::Coin, quantity),
            Content::Garbage(_) => backpack.add(backpack::BackpackContent::Garbage, quantity),
            Content::Fish(_) => backpack.add(backpack::BackpackContent::Fish, quantity),
            _ => panic!("Tried to insert a content that isn't in BackpackContent"),
        },
        MyEvents2::Event(Event::RemovedFromBackpack(content, quantity)) => match content {
            Content::Rock(_) => backpack.remove(backpack::BackpackContent::Rock, quantity),
            Content::Tree(_) => backpack.remove(backpack::BackpackContent::Tree, quantity),
            Content::Coin(_) => backpack.remove(backpack::BackpackContent::Coin, quantity),
            Content::Garbage(_) => backpack.remove(backpack::BackpackContent::Garbage, quantity),
            Content::Fish(_) => backpack.remove(backpack::BackpackContent::Fish, quantity),
            _ => panic!("Tried to remove a content that isn't in BackpackContent"),
        },
        MyEvents2::Event(Event::DayChanged(_))
        | MyEvents2::Event(Event::TimeChanged(_))
        | MyEvents2::Event(Event::EnergyRecharged(_))
        | MyEvents2::Event(Event::EnergyConsumed(_))
        | MyEvents2::Event(Event::TileContentUpdated(_, _))
        | MyEvents2::Event(Event::Ready)
        | MyEvents2::Event(Event::Terminated) => {
            panic!("Event {:?} shouldn't be here", next_event);
        }
    }
}

/// Handles keyboard inputs that are to be executed unconditionally
fn handle_inputs(
    my_camera: &mut Camera,
    robot_pos: &(usize, usize),
    time_manager: &mut TimeManager,
    show_menu: &mut bool,
) {
    if is_key_pressed(KeyCode::Escape) {
        exit(0);
    }
    if is_key_pressed(KeyCode::F) {
        my_camera.change_follow_robot(robot_pos);
    }

    if is_key_pressed(KeyCode::Left) {
        time_manager.change_speed(false);
    }
    if is_key_pressed(KeyCode::Right) {
        time_manager.change_speed(true);
    }
    if is_key_pressed(KeyCode::Tab) {
        *show_menu = !*show_menu;
    }
}

/// Writes to 2D camera useful info about the simulation
async fn write_debug(
    position: &Vec3,
    follow_robot: bool,
    direction: &Direction,
    backpack: &Backpack,
    current_speed: usize,
    events_left: f32,
    show_menu: bool,
) {
    set_default_camera();
    //TOP-LEFT corner
    let mut y = 20.;
    let offset = 30.;
    draw_text(
        &format!(
            "X: {:.2}, Y: {:.2}, Z: {:.2}",
            position.x, position.y, position.z
        ),
        10.,
        y,
        offset,
        BLACK,
    );
    y += offset;

    draw_text(&format!("Direction: {}", direction), 10., y, offset, BLACK);
    y += offset;
    draw_text(&format!("Fps: {}", get_fps()), 10., y, offset, BLACK);

    y += offset;
    let mode = if follow_robot {
        "Following"
    } else {
        "Free Camera"
    };
    draw_text(&format!("Current Mode: {}", mode), 10., y, offset, BLACK);

    //BOTTOM-LEFT corner
    y = screen_height() - offset + 5.;
    draw_text(
        &format!("Speed Level: {}", current_speed),
        10.,
        y,
        offset,
        BLACK,
    );

    y -= offset;
    draw_text(
        &format!("Total Done: {:.2}%", events_left),
        10.,
        y,
        offset,
        BLACK,
    );

    //TOP-RIGHT corner
    backpack.show().await;

    //BOTTOM-RIGHT corner
    if show_menu {
        ui::show().await;
    }
}

/// Draws the current map to the 3D Camera
pub(crate) async fn draw_world(
    world: &Vec<Vec<Option<Tile>>>,
    sprite_manager: &SpriteManager,
    position: &Vec3,
    direction: &Direction,
    robot_pos: (usize, usize),
) {
    //world
    for (x, row) in world.iter().enumerate() {
        for (y, tile) in row.iter().enumerate() {
            match tile {
                Some(tile) => match tile.tile_type {
                    TileType::Hill => sprite_manager.show(SpritesType::Hill, x, y, false),
                    TileType::Grass => sprite_manager.show(SpritesType::Grass, x, y, false),
                    TileType::Sand => sprite_manager.show(SpritesType::Sand, x, y, false),
                    TileType::Mountain => sprite_manager.show(SpritesType::Mountain, x, y, false),
                    TileType::Street => sprite_manager.show(SpritesType::Street, x, y, false),
                    TileType::Wall => sprite_manager.show(SpritesType::Wall, x, y, false),
                    TileType::ShallowWater => {
                        sprite_manager.show(SpritesType::ShallowWater, x, y, false)
                    }
                    TileType::DeepWater => sprite_manager.show(SpritesType::DeepWater, x, y, false),
                    TileType::Snow => sprite_manager.show(SpritesType::Snow, x, y, false),
                    TileType::Lava => sprite_manager.show(SpritesType::Lava, x, y, false),
                    TileType::Teleport(_) => {
                        sprite_manager.show(SpritesType::Teleport, x, y, false)
                    }
                },
                Option::None => {}
            }
        }
    }

    //contents
    draw_all_content(&world, &direction, &position, &sprite_manager, robot_pos);
}

/// Draw all contents to the 3D camera.
fn draw_all_content(
    world: &Vec<Vec<Option<Tile>>>,
    direction: &Direction,
    position: &Vec3,
    sprite_manager: &SpriteManager,
    robot_pos: (usize, usize),
) {
    //always render the world from far-away to close, to prevent visual issues
    let position = vec3(position.x, position.y, 1.);
    match *direction {
        Direction::North => {
            for y in (0..world.len()).into_iter().rev() {
                for x in 0..world.len() {
                    if x == robot_pos.0 && y == robot_pos.1 {
                        sprite_manager.show(SpritesType::Robot, robot_pos.0, robot_pos.1, true);
                    } else {
                        let tile = &world[x][y];
                        draw_content(tile, x, y, true, &sprite_manager, &position, robot_pos);
                    }
                }
            }
        }
        Direction::East => {
            for (x, row) in world.iter().enumerate().rev() {
                for (y, tile) in row.iter().enumerate() {
                    if x == robot_pos.0 && y == robot_pos.1 {
                        sprite_manager.show(SpritesType::Robot, x, y, false);
                    } else {
                        draw_content(tile, x, y, false, &sprite_manager, &position, robot_pos);
                    }
                }
            }
        }
        Direction::South => {
            for y in 0..world.len() {
                for x in 0..world.len() {
                    if x == robot_pos.0 && y == robot_pos.1 {
                        sprite_manager.show(SpritesType::Robot, robot_pos.0, robot_pos.1, true);
                    } else {
                        let tile = &world[x][y];
                        draw_content(tile, x, y, true, &sprite_manager, &position, robot_pos);
                    }
                }
            }
        }
        Direction::Ovest => {
            for (x, row) in world.iter().enumerate() {
                for (y, tile) in row.iter().enumerate() {
                    if x == robot_pos.0 && y == robot_pos.1 {
                        sprite_manager.show(SpritesType::Robot, x, y, false);
                    } else {
                        draw_content(tile, x, y, false, &sprite_manager, &position, robot_pos);
                    }
                }
            }
        }
    };
}

/// Calls the correct function to show each content. If the content is inside a radius of the robot or the camera
fn draw_content(
    tile: &Option<Tile>,
    x: usize,
    y: usize,
    facing_front: bool,
    sprite_manager: &SpriteManager,
    position: &Vec3,
    robot_pos: (usize, usize),
) {
    let human_distance = position.distance(vec3(x as f32, y as f32, 1.)) as i32;
    let robot_distance = vec3(robot_pos.0 as f32, robot_pos.1 as f32, 1.)
        .distance(vec3(x as f32, y as f32, 0.)) as i32;
    if human_distance > 70 && robot_distance > 60 {
        return;
    }

    match tile {
        Some(tile) => {
            #[rustfmt::skip]
                match tile.content {
                    Content::Rock(_) => sprite_manager.show(SpritesType::Rock, x, y, facing_front),
                    Content::Tree(_) => sprite_manager.show(SpritesType::Tree, x, y, facing_front),
                    Content::Coin(_) => sprite_manager.show(SpritesType::Coin, x, y, facing_front),
                    Content::Bank(_) => sprite_manager.show(SpritesType::Bank, x, y, facing_front),
                    Content::Fish(_) => sprite_manager.show(SpritesType::Fish, x, y, facing_front),
                    Content::Fire => sprite_manager.show(SpritesType::Fire, x, y, facing_front),
                    Content::Bin(_) => sprite_manager.show(SpritesType::Bin, x, y, facing_front),
                    Content::Garbage(_) => sprite_manager.show(SpritesType::Garbage, x, y, facing_front),
                    Content::Market(_) => sprite_manager.show(SpritesType::Market, x, y, facing_front),
                    Content::Bush(_) => sprite_manager.show(SpritesType::Bush, x, y, facing_front),
                    Content::Crate(_) => sprite_manager.show(SpritesType::Crate, x, y, facing_front),
                    Content::Water(_) => sprite_manager.show(SpritesType::Water, x, y, facing_front),
                    Content::Building => sprite_manager.show(SpritesType::Building, x, y, facing_front),
                    Content::JollyBlock(_) => sprite_manager.show(SpritesType::JollyBlock, x, y, facing_front),
                    Content::Scarecrow => sprite_manager.show(SpritesType::Scarecrow, x, y, facing_front),
                    Content::None => {}
                };
        }
        Option::None => {}
    };
}
