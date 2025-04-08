use robotics_lib::{
    self,
    world::{
        environmental_conditions::{EnvironmentalConditions, WeatherType},
        tile::{Content as RoboticLibContent, Tile, TileType},
        world_generator::Generator,
    },
};
/// Module with the test case in the showcase without biometype selected.
use std::collections::HashMap;
fn grass(elevation: usize, content: RoboticLibContent) -> Tile {
    Tile {
        tile_type: TileType::Grass,
        content,
        elevation,
    }
}
fn wall(elevation: usize) -> Tile {
    Tile {
        tile_type: TileType::Wall,
        content: RoboticLibContent::None,
        elevation,
    }
}
fn street(elevation: usize, content: RoboticLibContent) -> Tile {
    Tile {
        tile_type: TileType::Street,
        content,
        elevation,
    }
}
fn mountain(elevation: usize, content: RoboticLibContent) -> Tile {
    Tile {
        tile_type: TileType::Mountain,
        content,
        elevation,
    }
}
fn hill(elevation: usize, content: RoboticLibContent) -> Tile {
    Tile {
        tile_type: TileType::Hill,
        content,
        elevation,
    }
}
fn deep_water(elevation: usize) -> Tile {
    Tile {
        tile_type: TileType::DeepWater,
        content: RoboticLibContent::None,
        elevation,
    }
}
fn shallow_water(elevation: usize, content: RoboticLibContent) -> Tile {
    Tile {
        tile_type: TileType::ShallowWater,
        content,
        elevation,
    }
}
fn teleport(elevation: usize, state: bool) -> Tile {
    Tile {
        tile_type: TileType::Teleport(state),
        content: RoboticLibContent::None,
        elevation,
    }
}

pub struct World10X10;
#[allow(dead_code)]
impl World10X10 {
    pub fn new() -> Self {
        World10X10 {}
    }
}
const ROCKS: usize = 5;
const TREE: usize = 5;
const GARBAGE: usize = 5;
const FISH: usize = 2;
const COIN: usize = 2;
const DEPOSIT: usize = 100;

const CONTENT: [RoboticLibContent; 11] = [
    RoboticLibContent::Rock(ROCKS),
    RoboticLibContent::Tree(TREE),
    RoboticLibContent::Tree(TREE),
    RoboticLibContent::Tree(TREE),
    RoboticLibContent::Garbage(GARBAGE),
    RoboticLibContent::Coin(COIN),
    RoboticLibContent::Rock(ROCKS),
    RoboticLibContent::Tree(TREE),
    RoboticLibContent::Coin(COIN),
    RoboticLibContent::Garbage(GARBAGE),
    RoboticLibContent::None,
];

const BUILDING: [RoboticLibContent; 2] = [
    RoboticLibContent::Bank(0..DEPOSIT),
    RoboticLibContent::Market(DEPOSIT),
];

impl Generator for World10X10 {
    fn gen(
        &mut self,
    ) -> (
        Vec<Vec<Tile>>,
        (usize, usize),
        EnvironmentalConditions,
        f32,
        Option<HashMap<RoboticLibContent, f32>>,
    ) {
        let mut map: Vec<Vec<Tile>> = Vec::new();
        let mut elevation = 40;

        //first row
        map.push(Vec::new());
        map[0].push(teleport(elevation, false));
        (0..9).for_each(|_| map[0].push(street(elevation, RoboticLibContent::None)));
        elevation -= 4;
        //second row
        map.push(Vec::new());
        (0..5).for_each(|i| map[1].push(grass(elevation, CONTENT[i].clone())));
        map[1].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|i| map[1].push(grass(elevation, CONTENT[i + 7].clone())));
        elevation -= 4;

        //third row
        map.push(Vec::new());
        (0..5).for_each(|i| map[2].push(grass(elevation, CONTENT[i].clone())));
        map[2].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|i| map[2].push(grass(elevation, CONTENT[i + 7].clone())));
        elevation -= 4;

        //fourth row
        map.push(Vec::new());
        (0..5).for_each(|_i| map[3].push(shallow_water(elevation, RoboticLibContent::Fish(FISH))));
        map[3].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|_i| map[3].push(mountain(elevation, RoboticLibContent::None)));
        elevation -= 4;

        //fifth row
        map.push(Vec::new());
        (0..5).for_each(|_i| map[4].push(mountain(elevation, RoboticLibContent::None)));
        map[4].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|_i| map[4].push(mountain(elevation, RoboticLibContent::None)));
        elevation -= 4;

        //sixth row
        map.push(Vec::new());
        (0..5).for_each(|_i| map[5].push(wall(elevation)));
        map[5].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|_i| map[5].push(hill(elevation, RoboticLibContent::None)));
        elevation -= 4;

        //seventh row
        map.push(Vec::new());
        (0..5).for_each(|_i| map[6].push(mountain(elevation, RoboticLibContent::None)));
        map[6].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|_i| map[6].push(mountain(elevation, RoboticLibContent::None)));
        elevation -= 4;

        //eighth row
        map.push(Vec::new());
        (0..5).for_each(|_i| map[7].push(mountain(elevation, RoboticLibContent::None)));
        map[7].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|_i| map[7].push(deep_water(elevation)));
        elevation -= 4;

        //ninth row
        map.push(Vec::new());
        (0..5).for_each(|_i| map[8].push(grass(elevation, RoboticLibContent::None)));
        map[8].push(street(elevation, RoboticLibContent::None));
        (0..4).for_each(|_i| map[8].push(mountain(elevation, RoboticLibContent::None)));
        elevation -= 4;

        //tenth row
        map.push(Vec::new());
        // (0..9).for_each(|i| map[9].push(street(elevation, RoboticLibContent::None)));
        map[9].push(street(elevation, BUILDING[0].clone()));
        map[9].push(street(elevation, BUILDING[1].clone()));
        (0..3).for_each(|_| map[9].push(street(elevation, RoboticLibContent::None)));
        map[9].push(teleport(elevation, false));
        (0..2).for_each(|_| map[9].push(street(elevation, RoboticLibContent::None)));
        map[9].push(street(elevation, BUILDING[0].clone()));
        map[9].push(street(elevation, BUILDING[1].clone()));

        let environmental_conditions =
            EnvironmentalConditions::new(&[WeatherType::Sunny, WeatherType::Rainy], 15, 12);
        (map, (0, 0), environmental_conditions.unwrap(), 100.0, None)
    }
}
