use self::{robot::RobotTag, simulation_data::WorldSize};
use super::*;
use crate::global::*;
/// This module contains the camera and windows functionality for the visualizer.
/// It defines the camera components, systems, and plugins used to control the camera in the visualizer.
///
/// I used two windows with one camera each:
///     - `MapCamera`: will be on the left side and will show the map, it will change position if the robot exit the `RADIUS_FOLLOW_ROBOT`
///     - `UICamera`: will be on the right side and will show the data and some UI components
///
/// Useful info about Bevy 2D:
/// The X axis goes from left to right (+X points right), the Y axis goes from bottom to top (+Y points up),
/// and the Z axis goes from far to near. For 2D, the origin is at the center of the screen by default.
///
/// Use transform to position and rotate the camera of the visualizer.
/// Use projection to zoom.
/// With cameras you can use either have:
///     - OrthographicProjection: all entity are the same size even if they are futher away from the camera (2D, 3D)
///     - PerspectiveProjection: things appear smaller the further away they are from the camera (only 3D)  
///
///
use bevy::{
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
    winit::WinitWindows,
};

pub struct WindowsPlugin;

impl Plugin for WindowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT)
                        .with_scale_factor_override(1.0),
                    position: WindowPosition::At(IVec2::new(0, 0)),
                    resizable: false,
                    title: "Visualizer-1".to_owned(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Update, adjust_window_by_monitor_size)
        .add_systems(
            OnExit(SimulationState::Loading),
            build_map_window_camera.run_if(in_state(SimulationState::Simulation)),
        )
        .add_systems(
            Update,
            zoom_scale.run_if(in_state(SimulationState::Simulation)),
        )
        .add_systems(
            PostUpdate,
            follow_the_robot.run_if(in_state(SimulationState::Simulation)),
        )
        .add_systems(OnEnter(SimulationState::Simulation), spawn_ui_camera);
    }
}

/// Camera for the map.
#[derive(Component)]
pub struct MapCamera;
/// Window for the map.
#[derive(Component)]
pub struct MapWindow;
/// Camera for the user interface nodes.
#[derive(Component)]
pub struct UICamera;

/// System that spawn the window that will contain the user interface nodes.
fn build_map_window_camera(mut commands: Commands, world_size: Res<WorldSize>) {
    let map_window = commands
        .spawn((
            Window {
                title: "Map".to_owned(),
                // position the window just after the UI window
                position: WindowPosition::At(IVec2::new(UI_BAR as i32, 0)),
                resolution: WindowResolution::new(MAP_WINDOWW_WIDTH, MAP_WINDOWW_HEIGHT),
                resizable: false,
                ..default()
            },
            // Window Tag
            MapWindow,
        ))
        .id();

    let middle = (world_size.0 / 2) as f32;
    // Camera to visualize the map and the robot.
    commands.spawn((
        Camera2dBundle {
            // Change the origin from the center to the bottom left corner.
            transform: Transform::from_xyz(middle * TILE_SIZE, middle * TILE_SIZE, 0.0),
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(map_window)),
                ..default()
            },
            ..default()
        },
        // Don't shwo user interface nodes in the map camera.
        UiCameraConfig { show_ui: false },
        MapCamera,
    ));
}

/// System that will keep the robot in sight.
#[allow(clippy::type_complexity)]
fn follow_the_robot(
    mut camera: Query<
        (&mut Transform, &OrthographicProjection),
        (With<MapCamera>, Without<RobotTag>),
    >,
    robot: Query<&Transform, With<RobotTag>>,
) {
    // Take the position of the Robot.
    let transform = robot.single();

    // Take the position of the camera with the projection properties.
    let (mut transform_camera, projection) = camera.single_mut();

    // Need to keep in consideration the zooming of the camera. If the robot is outside the
    // radius then realign his position according to the robot position.
    if (transform.translation.x - transform_camera.translation.x).abs()
        > RADIUS_FOLLOW_ROBOT * projection.scale
        || (transform.translation.y - transform_camera.translation.y).abs()
            > RADIUS_FOLLOW_ROBOT * projection.scale
    {
        transform_camera.translation.x = transform.translation.x;
        transform_camera.translation.y = transform.translation.y;
    }
}

// System that take keyboard input and change the zooming of the map.
fn zoom_scale(
    mut query_camera: Query<&mut OrthographicProjection, With<MapCamera>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut projection = query_camera.single_mut();

    if keyboard_input.pressed(KeyCode::Space) && projection.scale > 0.25 {
        projection.scale /= 1.05;
    }
    if keyboard_input.pressed(KeyCode::Minus) && projection.scale < 5.0 {
        projection.scale *= 1.05;
    }
}

fn spawn_ui_camera(mut commands: Commands) {
    // Camera to visualize the UI Bar.
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.),
            camera: Camera {
                target: bevy::render::camera::RenderTarget::Window(WindowRef::Primary),
                ..default()
            },
            ..default()
        },
        UICamera,
    ));
}

/// This system takes info about the actual monitor and resize the window accordingly,
/// then in the menu system it will resize the nodes to fit them in the screen.
fn adjust_window_by_monitor_size(
    winit_windows: NonSend<WinitWindows>,
    mut window_query_menu: Query<(Entity, &mut Window), (With<PrimaryWindow>, Without<MapWindow>)>,
    mut window_query_map: Query<&mut Window, (With<MapWindow>, Without<PrimaryWindow>)>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    // take the UI window
    let main = window_query_menu.get_single_mut();
    // take the map window
    let mappa = window_query_map.get_single_mut();
    // if the UI window is open
    if let Ok((id_menu, mut menu)) = main {
        // take the monitor handle of the UI window
        if let Some(monitor) = winit_windows
            .get_window(id_menu)
            .and_then(|winit_window| winit_window.current_monitor())
        {
            let width = monitor.size().width;
            let height = monitor.size().height;
            // resize the window in order to fit
            menu.resolution = WindowResolution::new(width as f32 * 0.333, height as f32 * 0.9);

            //  place the map window just next to the UI window
            if let Ok(mut m) = mappa {
                m.position = WindowPosition::At(IVec2::new((width as f32 * 0.333) as i32 + 5, 0));
            }
        }
    // If the UI window is closed exit the APP
    } else {
        app_exit_events.send(bevy::app::AppExit);
    }
}
