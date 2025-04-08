use super::style::*;
use crate::{
    asset_loader::{
        CustomRobotImages, DefaultRobotImages, RobotPath, CUSTOM_ROBOT_PATH, DEFAULT_ROBOT_PATH,
    },
    button_text_style,
    music::AmbientMusic,
    robot::MoveRobotTimer,
    showcase::{self, WORLD_SIZE_BIOME, WORLD_SIZE_10X10},
    simulation_data::WorldSize,
    spawn_animation_button, spawn_button, spawn_button_showcase, spawn_container_node,
    spawn_heading_node, spawn_setting_value_node, spawn_sub_container_node,
    sprite_animation::AnimationTimer,
    training_values::TrainingValues,
    windows_cameras::{MapCamera, MapWindow},
    SimulationState,
};
use ai::ai::REWARDS;
/// This file holds all the different screens of the user iterface before the simulation begin.
/// So the Menu has different state corresponding to different screen:
/// - Main Menu: from here the user can either start the simulation or go to the other screen/state of the menu
/// - Setting Menu: set the size of the world, the robot to be used
/// - Training Menu: change the default value of the Q-learning algorithm of the AI in order to train a custom AI
/// - Showcase Menu: from here you can check out the Planner Tool and World Generator
///
///
use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use rustici_world_generator::biomes::BiomeType;
use std::{fs::File, io::Write, time::Duration};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Showcase>()
            .add_state::<SimulationState>()
            .add_state::<MenuState>()
            .add_systems(OnEnter(SimulationState::Menu), menu_setup)
            .add_systems(Startup, menu_camera)
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(
                Update,
                (animate_button, select_bot_button).run_if(in_state(MenuState::Settings)),
            )
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnSettingsMenuScreen>,
            )
            .add_systems(OnEnter(MenuState::Training), setting_menu_training)
            .add_systems(
                OnExit(MenuState::Training),
                despawn_screen::<OnTrainingScreen>,
            )
            .add_systems(Update, menu_action)
            .add_systems(Update, button_system)
            .add_systems(
                Update,
                select_volume_button.run_if(in_state(SimulationState::Simulation)),
            )
            .add_systems(
                OnExit(SimulationState::Menu),
                (despawn_screen::<OnMainMenuScreen>, loading_setup),
            )
            .add_systems(
                Update,
                run_simulation.run_if(in_state(SimulationState::Loading)),
            )
            .add_systems(
                OnExit(SimulationState::Loading),
                (
                    despawn_screen::<MenuCamera>,
                    despawn_screen::<OnLoadingScreen>,
                ),
            )
            .add_systems(OnEnter(MenuState::Showcase), setup_showcase_screen)
            .add_systems(
                OnExit(MenuState::Showcase),
                despawn_screen::<OnShowcaseScreen>,
            )
            .add_systems(
                OnExit(SimulationState::Menu),
                despawn_screen::<OnShowcaseScreen>,
            )
            .add_systems(
                Update,
                select_biome_button.run_if(in_state(MenuState::Showcase)),
            )
            .add_systems(OnEnter(SimulationState::Training), train_user_robot);
    }
}

#[derive(Component)]
pub struct MenuCamera;
fn menu_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MenuCamera));
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Main,
    Settings,
    Training,
    Showcase,
}

// Tag components used to easily retreive Entity in systems
#[derive(Component)]
pub(crate) struct RealTimeText(String);
#[derive(Component)]
struct OnMainMenuScreen;
#[derive(Component)]
struct OnSettingsMenuScreen;
#[derive(Component)]
struct OnTrainingScreen;
#[derive(Component)]
struct OnShowcaseScreen;
#[derive(Component)]
struct OnLoadingScreen;
#[derive(Component)]
struct LoadingText;
// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
pub(crate) struct AnimateButton {
    pub(crate) index: usize,
    pub(crate) total: usize,
    pub(crate) default: bool,
}
impl AnimateButton {
    fn new(total: usize, default: bool) -> Self {
        AnimateButton {
            index: 0,
            total,
            default,
        }
    }
}
#[derive(Component)]
pub struct AnimatedButton;
#[derive(Component)]
pub struct ButtonShowcase;
// All actions that can be triggered from a button click
#[derive(Component, Eq, PartialEq)]
pub(crate) enum MenuButtonAction {
    Start,
    BackToMainMenu,
    Quit,

    Settings,
    Size(Change),
    DefaultBot,
    CustomBot,

    Training,
    ExitAndTrain,
    Rewards(usize, String, Change),

    Showcase,
    Test,
    Biome(BiomeType),

    Music,
    Mute,

    Zoom(Change),
    Speed(Change),
}
// Helper function to translate an action to a BiomeType used in the showcase screen
impl MenuButtonAction {
    fn is_biome(&self, biome: &BiomeType) -> bool {
        if let MenuButtonAction::Biome(b) = self {
            *b == *biome
        } else {
            false
        }
    }
}
#[derive(Component, Eq, PartialEq, Debug)]
pub(crate) enum Change {
    Up,
    Down,
}

// Timer needed to show first the Loading screen and then call the AI.
#[derive(Resource, Deref, DerefMut)]
pub struct LoadingTimer(pub Timer);

impl Default for LoadingTimer {
    fn default() -> Self {
        LoadingTimer(Timer::new(Duration::from_millis(500), TimerMode::Once))
    }
}

#[derive(Resource, Debug, Default)]
pub struct Showcase {
    set: bool,
    biome_type: Option<BiomeType>,
}

/// This system will run the simulation after the user press Start.
fn run_simulation(
    time: Res<Time>,
    world_size: Res<WorldSize>,
    mut loading_timer: ResMut<LoadingTimer>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
    showcase: Res<Showcase>,
    robot_path: Res<RobotPath>,
) {
    loading_timer.tick(time.delta());
    if loading_timer.just_finished() {
        // run the rustici tool showcase
        if showcase.set {
            match showcase.biome_type {
                // If biome is selected will run the test with our world generator 30x30 
                Some(_) => showcase::run(WORLD_SIZE_BIOME, showcase.biome_type),
                // If no biome is selected it will run on a simple hardcoded world 10x10
                None => showcase::run(WORLD_SIZE_10X10, showcase.biome_type),
            }
        // run AI simulation
        } else {
            ai::ai::ai(world_size.0, REWARDS.to_vec(), robot_path.is_default());
        }
        simulation_state.set(SimulationState::Simulation);
    }
}

/// System that save the user reward values into a file and exit from the Bevy App
/// The file will be read from the entry_point.rs and there will start the training
/// of the customized bot.
fn train_user_robot(
    training_values: ResMut<TrainingValues>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    let serialized = ron::to_string(&training_values.0).expect("Serialization failed.");
    let mut file = File::create("rewards.ron").expect("Failed to create file.");
    file.write_all(serialized.as_bytes())
        .expect("Failed to write to file.");
    app_exit_events.send(AppExit);
}

/// System that prepare loading window.
fn loading_setup(mut commands: Commands) {
    let container = spawn_container_node![commands];
    let sub_container = spawn_sub_container_node![commands, Color::SILVER];

    let text = spawn_heading_node!(
        commands,
        "Loading, please wait...",
        50.0,
        200.0,
        JustifyContent::Center
    );

    commands.entity(sub_container).add_child(text);
    commands.entity(container).add_child(sub_container);
    commands.init_resource::<LoadingTimer>();
}

/// System that handles button colors change.
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (
            Changed<Interaction>,
            With<Button>,
            Without<AnimateButton>,
            Without<ButtonShowcase>,
        ),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

/// Set up the Main menu state.
fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

// System that lays out the main menu.
fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let container = spawn_container_node![commands, OnMainMenuScreen];
    let sub_container = spawn_sub_container_node![commands, Color::SILVER];

    let heading = spawn_heading_node![commands, "Rustici AI", 60., 130., JustifyContent::Center];
    let start_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Start,
        "menu_icons/start.png",
        "Start",
        130.
    ];
    let setting_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Settings,
        "menu_icons/setting.png",
        "Settings",
        130.
    ];
    let training_setup_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Training,
        "menu_icons/training.png",
        "Training",
        130.
    ];
    let rustici_showcase = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Showcase,
        "menu_icons/showcase.png",
        "Rustici Tool & WG",
        130.
    ];
    let quit_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Quit,
        "menu_icons/exit.png",
        "Quit",
        130.
    ];

    commands.entity(sub_container).push_children(&[
        heading,
        start_button,
        setting_button,
        training_setup_button,
        rustici_showcase,
        quit_button,
    ]);
    commands.entity(container).add_child(sub_container);
}

/// System that prepare the settings layout.
fn settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_size: Res<WorldSize>,
    window: Query<&Window, (With<PrimaryWindow>, Without<MapWindow>)>,
    default_robot_handle: Res<DefaultRobotImages>,
    custom_robot_handle: Res<CustomRobotImages>,
) {
    let button_text_style = button_text_style![];

    let width = window.single().width();
    let container = spawn_container_node![commands, OnSettingsMenuScreen];

    let sub_container = spawn_sub_container_node![commands, Color::SILVER];
    let setting_heading =
        spawn_heading_node![commands, "Setting", 60.0, 130.0, JustifyContent::Center];
    let world_size_setting = spawn_setting_value_node![
        commands,
        width * 0.8,
        130.,
        "menu_icons/world.png",
        "Size:",
        button_text_style,
        asset_server,
        world_size.0,
        "+10",
        "-10",
        MenuButtonAction::Size(Change::Up),
        MenuButtonAction::Size(Change::Down),
        RealTimeText("world_size".to_string())
    ];

    let bot_button = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .id();
    let default_bot_button = spawn_animation_button![
        commands,
        default_robot_handle,
        MenuButtonAction::DefaultBot,
        "Default Robot",
        130.,
        300.,
        AnimateButton::new(8, true),
        AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating))
    ];
    let custom_bot_button = spawn_animation_button![
        commands,
        custom_robot_handle,
        MenuButtonAction::CustomBot,
        "Custom Robot",
        130.,
        300.,
        AnimateButton::new(8, false),
        AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating))
    ];

    commands
        .entity(bot_button)
        .push_children(&[default_bot_button, custom_bot_button]);

    let back_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::BackToMainMenu,
        "menu_icons/back.png",
        "Back",
        130.
    ];
    commands.entity(sub_container).push_children(&[
        setting_heading,
        world_size_setting,
        bot_button,
        back_button,
    ]);
    commands.entity(container).add_child(sub_container);
}

const REWARDS_NAME_LABEL: [(&str, &str); 8] = [
    ("start", "Start:"),
    ("goal", "Goal:"),
    ("destroy", "Destroyed:"),
    ("sold", "Sold:"),
    ("put_in_bank", "Put Coin:"),
    ("recycled", "Recycled:"),
    ("need_exploring", "Exploring:"),
    ("backpack_full", "Backpack Full:"),
];

/// System that prepare the user training layout.
fn setting_menu_training(
    mut commands: Commands,
    training_values: ResMut<TrainingValues>,
    window_q: Query<&Window, (With<PrimaryWindow>, Without<MapWindow>)>,
    asset_server: Res<AssetServer>,
) {
    let window = window_q.single();
    let width = window.resolution.width();
    // Number of nodes to compute the right height for each one relative to the monitor.
    const NODES: f32 = 15.;
    let window_height = window.height();
    let nodes_height = window_height / NODES;
    let container = spawn_container_node![commands, OnTrainingScreen];
    let sub_container = spawn_sub_container_node![commands, Color::SILVER];

    let setting_heading = spawn_heading_node![
        commands,
        "Q-learning reward values:",
        40.0,
        nodes_height,
        JustifyContent::Center
    ];

    commands.entity(sub_container).add_child(setting_heading);

    for (index, (name, label)) in REWARDS_NAME_LABEL.iter().enumerate() {
        let reward_setting = spawn_setting_value_node![
            commands,
            width * 0.9,
            nodes_height,
            format!("reward_icons/{}.png", name),
            *label,
            button_text_style![],
            asset_server,
            training_values.0[index],
            "+1",
            "-1",
            MenuButtonAction::Rewards(index, name.to_string(), Change::Up),
            MenuButtonAction::Rewards(index, name.to_string(), Change::Down),
            RealTimeText(name.to_string())
        ];
        commands.entity(sub_container).add_child(reward_setting);
    }

    let exit_and_train = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::ExitAndTrain,
        "menu_icons/start.png",
        "Exit and Train",
        nodes_height
    ];
    let back_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::BackToMainMenu,
        "menu_icons/back.png",
        "Back",
        nodes_height
    ];

    commands
        .entity(sub_container)
        .push_children(&[exit_and_train, back_button]);
    commands.entity(container).add_child(sub_container);
}

/// System that manage all the button action of the Visualizer.
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
    mut real_text_area: Query<(&mut Text, &RealTimeText)>,
    mut world_size: ResMut<WorldSize>,
    mut query_camera: Query<&mut OrthographicProjection, With<MapCamera>>,
    mut timer: ResMut<MoveRobotTimer>,
    mut training_values: ResMut<TrainingValues>,
    mut showcase: ResMut<Showcase>,
    mut robot_path: ResMut<RobotPath>,
    music_box_query: Query<&AudioSink, With<AmbientMusic>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        // controll only pressed interaction
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                // Start the normal simulation
                MenuButtonAction::Start => {
                    showcase.set = false;
                    simulation_state.set(SimulationState::Loading);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                // Setting: world size
                MenuButtonAction::Size(c) => {
                    match c {
                        Change::Up => {
                            if world_size.0 < 600 {
                                world_size.0 += 10;
                            }
                        }
                        Change::Down => {
                            if world_size.0 > 20 {
                                world_size.0 -= 10;
                            }
                        }
                    }
                    for (mut text, name) in real_text_area.iter_mut() {
                        if name.0 == "world_size" {
                            text.sections[0].value = world_size.0.to_string();
                        }
                    }
                }
                // Setting: choose pretrained bot
                MenuButtonAction::DefaultBot => {
                    if robot_path.0 == CUSTOM_ROBOT_PATH.to_string() {
                        robot_path.0 = DEFAULT_ROBOT_PATH.to_string();
                    }
                }
                // Setting: choose custom trained bot
                MenuButtonAction::CustomBot => {
                    if robot_path.0 == DEFAULT_ROBOT_PATH.to_string() {
                        robot_path.0 = CUSTOM_ROBOT_PATH.to_string();
                    }
                }
                // Simulation button: zoom
                MenuButtonAction::Zoom(c) => {
                    let mut projection = query_camera.single_mut();
                    match c {
                        Change::Up => {
                            if projection.scale > 0.25 {
                                projection.scale /= 1.2;
                            }
                        }
                        Change::Down => {
                            if projection.scale < 5.0 {
                                projection.scale *= 1.2;
                            }
                        }
                    }
                }
                // Simulation button: speed of simulation and music pitch
                MenuButtonAction::Speed(c) => {
                    let mut millis = timer.timer.duration().as_millis();
                    match c {
                        Change::Down => {
                            millis += 50;
                            if millis < 1600 {
                                timer
                                    .timer
                                    .set_duration(Duration::from_millis(millis as u64));
                                let sink = music_box_query.single();
                                let speed = sink.speed() - 0.025;
                                sink.set_speed(speed);
                            }
                        }
                        Change::Up => {
                            millis -= 50;
                            if millis > 10 {
                                timer
                                    .timer
                                    .set_duration(Duration::from_millis(millis as u64));
                                let sink = music_box_query.single();
                                let speed = sink.speed() + 0.025;
                                sink.set_speed(speed);
                            }
                        }
                    }
                }
                // Simulation: Music ON
                MenuButtonAction::Music => {
                    if let Ok(sink) = music_box_query.get_single() {
                        sink.set_volume(1.0);
                    }
                }
                // Simulation: Music OFF
                MenuButtonAction::Mute => {
                    if let Ok(sink) = music_box_query.get_single() {
                        sink.set_volume(0.0);
                    }
                }
                // Training
                MenuButtonAction::Training => menu_state.set(MenuState::Training),
                // Change custom reward value of the Q-learning algorithm.
                MenuButtonAction::Rewards(index, label, c) => {
                    match c {
                        Change::Up => {
                            training_values.0[*index] += 1.;
                        }
                        Change::Down => {
                            training_values.0[*index] -= 1.;
                        }
                    }
                    for (mut text, name) in real_text_area.iter_mut() {
                        if name.0 == *label {
                            text.sections[0].value = training_values.0[*index].to_string();
                        }
                    }
                }
                // Start the training of the custom bot.
                MenuButtonAction::ExitAndTrain => simulation_state.set(SimulationState::Training),
                // Showcase
                MenuButtonAction::Showcase => {
                    menu_state.set(MenuState::Showcase);
                }
                // Showcase: Run a test simulation of the planner and WG
                MenuButtonAction::Test => {
                    showcase.set = true;
                    simulation_state.set(SimulationState::Loading);
                }
                // Showcase: Choose the biometype
                MenuButtonAction::Biome(b) => {
                    showcase.biome_type = Some(*b);
                }
            }
        }
    }
}

/// System that lay out the showcase screen
fn setup_showcase_screen(
    mut commands: Commands,
    window: Query<&Window, (With<PrimaryWindow>, Without<MapWindow>)>,
    asset_server: Res<AssetServer>,
) {
    const NODES: f32 = 15.;
    let window_height = window.single().height();
    let nodes_height = window_height / NODES;
    let container = spawn_container_node![commands, OnShowcaseScreen];

    let sub_container = spawn_sub_container_node![commands, Color::SILVER];
    let tool_heading = spawn_heading_node![
        commands,
        "Tool & WG",
        45.0,
        nodes_height,
        JustifyContent::Center
    ];

    let run_biome_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Test,
        "biome_icons/test.png",
        "Test!",
        nodes_height
    ];
    let back_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::BackToMainMenu,
        "menu_icons/back.png",
        "Back",
        nodes_height
    ];
    let biome_heading = spawn_heading_node![
        commands,
        "Select Biome:",
        40.0,
        nodes_height,
        JustifyContent::Center
    ];

    let plain_biome_button = spawn_button_showcase![
        commands,
        asset_server,
        MenuButtonAction::Biome(BiomeType::Plain),
        "biome_icons/plain.png",
        "Plain",
        nodes_height
    ];
    let marine_biome_button = spawn_button_showcase![
        commands,
        asset_server,
        MenuButtonAction::Biome(BiomeType::Marine),
        "biome_icons/marine.png",
        "Marine",
        nodes_height
    ];
    let mountain_biome_button = spawn_button_showcase![
        commands,
        asset_server,
        MenuButtonAction::Biome(BiomeType::Mountains),
        "biome_icons/mountain.png",
        "Mountain",
        nodes_height
    ];
    let hell_biome_button = spawn_button_showcase![
        commands,
        asset_server,
        MenuButtonAction::Biome(BiomeType::Hell),
        "biome_icons/hell.png",
        "Hell",
        nodes_height
    ];
    let glacial_biome_button = spawn_button_showcase![
        commands,
        asset_server,
        MenuButtonAction::Biome(BiomeType::Glacial),
        "biome_icons/glacial.png",
        "Glacial",
        nodes_height
    ];

    commands.entity(sub_container).push_children(&[
        tool_heading,
        run_biome_button,
        back_button,
        biome_heading,
        plain_biome_button,
        mountain_biome_button,
        marine_biome_button,
        hell_biome_button,
        glacial_biome_button,
    ]);
    commands.entity(container).add_child(sub_container);
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub(crate) fn despawn_screen<T: Component>(
    to_despawn: Query<Entity, With<T>>,
    mut commands: Commands,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
/// System for animating the setting button to choose the bot for the simulation.
fn animate_button(
    mut button_query: Query<(&mut UiImage, &mut AnimateButton, &mut AnimationTimer)>,
    default_imgs: Res<DefaultRobotImages>,
    custom_imgs: Res<CustomRobotImages>,
    time: Res<Time>,
) {
    for (mut image, mut animate_button, mut timer) in button_query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            let index = (animate_button.index + 1) % animate_button.total;
            if animate_button.default {
                *image = default_imgs.0[index].clone().into()
            } else {
                *image = custom_imgs.0[index].clone().into()
            }
            animate_button.index = index;
        }
    }
}
/// System that keep selected the bot button on setting screen.
fn select_bot_button(
    mut button_query: Query<
        (&mut BackgroundColor, &mut BorderColor, &MenuButtonAction),
        With<AnimatedButton>,
    >,
    robot_path: ResMut<RobotPath>,
) {
    for (mut color, mut border_color, action) in button_query.iter_mut() {
        if (robot_path.0 == DEFAULT_ROBOT_PATH.to_string()
            && *action == MenuButtonAction::DefaultBot)
            || robot_path.0 == CUSTOM_ROBOT_PATH.to_string()
                && *action == MenuButtonAction::CustomBot
        {
            *color = HOVERED_BUTTON.into();
            *border_color = Color::BLACK.into();
        } else {
            *color = NORMAL_BUTTON.into();
            *border_color = NORMAL_BUTTON.into();
        }
    }
}

/// System that keep selected the biome type button.
fn select_biome_button(
    mut button_query: Query<
        (&mut BackgroundColor, &mut BorderColor, &MenuButtonAction),
        With<ButtonShowcase>,
    >,
    showcase: Res<Showcase>,
) {
    if let Some(biome_type) = showcase.biome_type {
        for (mut color, mut border_color, action) in button_query.iter_mut() {
            if action.is_biome(&biome_type) {
                *color = HOVERED_BUTTON.into();
                *border_color = Color::BLACK.into();
            } else {
                *color = NORMAL_BUTTON.into();
                *border_color = NORMAL_BUTTON.into();
            }
        }
    }
}
/// System that keep selected the music button on simulation state.
fn select_volume_button(
    mut button_query: Query<(&mut BackgroundColor, &MenuButtonAction)>,
    music_box_query: Query<&AudioSink, With<AmbientMusic>>,
) {
    if let Ok(sink) = music_box_query.get_single() {
        for (mut color, action) in button_query.iter_mut() {
            if let MenuButtonAction::Music = action {
                if sink.volume() != 0.0 {
                    *color = HOVERED_BUTTON.into();
                } else {
                    *color = Color::WHITE.into();
                }
            }
            if let MenuButtonAction::Mute = action {
                if sink.volume() != 0.0 {
                    *color = Color::WHITE.into();
                } else {
                    *color = HOVERED_BUTTON.into();
                }
            }
        }
    }
}
