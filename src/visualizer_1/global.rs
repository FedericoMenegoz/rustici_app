/**************** VISUALIZER 1 ****************/
// Main window and UI bar
pub(crate) const WINDOW_WIDTH: f32 = 2870.;
pub(crate) const WINDOW_HEIGHT: f32 = 1700.;
pub(crate) const UI_BAR: f32 = 800.;
pub(crate) const MAP_WINDOWW_WIDTH: f32 = 800.;
pub(crate) const MAP_WINDOWW_HEIGHT: f32 = 800.;

pub(crate) const RADIUS_FOLLOW_ROBOT: f32 = 380.;

// Robot moves interval.
pub(crate) const MOVE_TIMER_SECONDS: f32 = 0.8;

pub(crate) const TILE_SIZE: f32 = 32.0;

// Stack order for object to render.
pub(crate) const TILE_Z_INDEX: f32 = 0.0;
pub(crate) const CONTENT_Z_INDEX: f32 = 1.0;
pub(crate) const ROBOT_Z_INDEX: f32 = 2.0;

/************************************************/

/******************* AI DATA ********************/
// AI data
pub(crate) const WORLD_SIZE: usize = 200;
pub(crate) const MAX_ENERGY_LEVEL: usize = 1000;
// Default values for the reward function