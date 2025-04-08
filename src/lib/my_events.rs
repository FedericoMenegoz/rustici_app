use robotics_lib::{
    event::events::Event,
    world::tile::{Content, Tile},
};

#[derive(Debug)]
pub enum MyEvents2 {
    Event(Event),
    RobotSpawned((usize, usize)),
    UsedTool(Vec<Vec<Option<Tile>>>),
    ContentInteracted(Content, (usize, usize)),
}
