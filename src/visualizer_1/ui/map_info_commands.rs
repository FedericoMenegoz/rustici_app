/// This is the user interface to show the info during the visualization of the simulation.
/// It will hold:
/// - the energy level
/// - the backpack content
/// - the amount of coin to reach the goal and how many has been deposited
/// - commands button do control the velocity of the simulation, zooming and music
///
/// I had to use two different window for the map and the UI as for now the UI nodes in bevy create some issue if mixed with world entities.
use self::simulation_data::{SimulationData, TotalTransactions, Transaction};
use super::menu::{despawn_screen, MenuButtonAction};
use super::style::*;
use crate::{
    simulation_data::{backpack::MyBackPack, energy::MyEnergy, AvailableContent, CoinsToDeposit},
    spawn_box_node, spawn_button, spawn_container_node, spawn_heading_node,
    spawn_sub_container_node,
    ui::menu::Change,
    *,
};
use ai::data_storage::MyEvent;
use bevy::window::PrimaryWindow;
use robotics_lib::{
    event::events::Event as RoboticLibEvent, world::tile::Content as RoboticLibContent,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Simulation), setup_info_simulation)
            .add_systems(
                OnExit(SimulationState::Simulation),
                despawn_screen::<OnSimulationInfoScreen>,
            )
            .add_systems(OnEnter(SimulationState::Result), setup_info_result)
            .add_systems(
                Update,
                (show_simulation_data, check_if_simulation_is_terminated)
                    .run_if(in_state(SimulationState::Simulation)),
            );
    }
}

// Tags to easily get component in Bevy Queries.
#[derive(Debug, Component)]
pub struct EnergyValueNode;
#[derive(Debug, Component)]
pub struct CoinsToDepositNode;
#[derive(Debug, Component)]
pub struct MarketNode;
#[derive(Debug, Component)]
pub struct BankNode;

#[derive(Debug, Component)]
pub struct ZoomInButton;
#[derive(Debug, Component)]
pub struct ZoomOutButton;

#[derive(Debug, Component, Copy, Clone)]
pub enum ContentNode {
    RockNode,
    WoodNode,
    FishNode,
    GarbageNode,
    CoinNode,
}

const BACKPACK_CONTENT: &[(ContentNode, &str); 5] = &[
    (ContentNode::RockNode, "ui_icons/rock.png"),
    (ContentNode::WoodNode, "ui_icons/wood.png"),
    (ContentNode::FishNode, "ui_icons/fish.png"),
    (ContentNode::GarbageNode, "ui_icons/garbage.png"),
    (ContentNode::CoinNode, "ui_icons/coin.png"),
];

#[derive(Component)]
pub(crate) struct OnSimulationInfoScreen;
#[derive(Component)]
pub(crate) struct OnResultScreen;

/// System that setup the layout of the UI
fn setup_info_simulation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    energy: Res<MyEnergy>,
    backpack: Res<MyBackPack>,
    coin_to_deposit: Res<CoinsToDeposit>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    // Number of nodes to compute the right height for each one relative to the monitor.
    const NODES: f32 = 12.;
    let mut content_to_push = Vec::new();

    let window_height = window.single().height();
    let box_height = window_height / NODES;
    let reduced_height = box_height * 3. / 4.;
    // Spawn the screen component.
    let container = spawn_container_node![commands, OnSimulationInfoScreen];
    let sub_container = spawn_sub_container_node![commands, Color::ANTIQUE_WHITE];

    // Energy
    let energy_heading = spawn_heading_node![
        commands,
        "Energy:",
        50.0,
        reduced_height,
        JustifyContent::FlexStart
    ];
    let energy_box = spawn_box_node![
        commands,
        asset_server,
        "ui_icons/energy.png",
        energy.0,
        EnergyValueNode,
        box_height
    ];
    // Backpack
    let heading_backpack = spawn_heading_node![
        commands,
        "Backpack:",
        50.0,
        box_height,
        JustifyContent::FlexStart
    ];

    content_to_push.extend_from_slice(&[energy_heading, energy_box, heading_backpack]);

    for (content_tag, path) in BACKPACK_CONTENT.iter() {
        let val = if let Some(val) = backpack.0.get(&match_node_content(content_tag)) {
            val.to_string()
        } else {
            0.to_string()
        };
        let content_box =
            spawn_box_node![commands, asset_server, *path, val, *content_tag, box_height];
        content_to_push.push(content_box);
    }
    // Goal
    let heading_goal_val = format!("Goal [{}]:", coin_to_deposit.total);
    let coin_to_deposit_heading = spawn_heading_node![
        commands,
        heading_goal_val,
        50.0,
        box_height,
        JustifyContent::FlexStart
    ];
    let coin_to_deposit_box = spawn_box_node![
        commands,
        asset_server,
        "ui_icons/piggy.png",
        "0",
        CoinsToDepositNode,
        box_height
    ];
    // Controls
    // Zoom
    let buttons_box_zoom = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                margin: UiRect::top(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                height: Val::Px(reduced_height),
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();

    let zoom_in_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Zoom(Change::Up),
        "ui_icons/zoom_in.png",
        80.
    ];
    let zoom_out_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Zoom(Change::Down),
        "ui_icons/zoom_out.png",
        80.
    ];
    // Sound
    let music_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Music,
        "ui_icons/volume_up.png",
        80.
    ];
    let mute_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Mute,
        "ui_icons/mute.png",
        80.
    ];
    // Speed
    let buttons_box_speed = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                height: Val::Px(box_height),
                // border: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();

    let speed_up_button = spawn_button!(
        commands,
        MenuButtonAction::Speed(Change::Up),
        "Speed Up",
        80.
    );

    let speed_down_button = spawn_button!(
        commands,
        MenuButtonAction::Speed(Change::Down),
        "Speed Down",
        80.
    );
    commands.entity(buttons_box_zoom).push_children(&[
        zoom_in_button,
        zoom_out_button,
        music_button,
        mute_button,
    ]);
    commands
        .entity(buttons_box_speed)
        .push_children(&[speed_up_button, speed_down_button]);
    content_to_push.extend_from_slice(&[
        coin_to_deposit_heading,
        coin_to_deposit_box,
        buttons_box_zoom,
        buttons_box_speed,
    ]);
    commands
        .entity(sub_container)
        .push_children(&content_to_push);
    commands.entity(container).add_child(sub_container);
}

/// System that keep updated the info in the UI.
fn show_simulation_data(
    mut testo_e: Query<&mut Text, (With<EnergyValueNode>, Without<CoinsToDepositNode>)>,
    mut testo_c: Query<&mut Text, (With<CoinsToDepositNode>, Without<EnergyValueNode>)>,
    mut backpack_q: Query<
        (&mut Text, &ContentNode),
        (Without<EnergyValueNode>, Without<CoinsToDepositNode>),
    >,
    energy: Res<MyEnergy>,
    backpack: Res<MyBackPack>,
    _available_content: Res<AvailableContent>,
    coins_to_deposit: Res<CoinsToDeposit>,
) {
    // Energy value
    let mut testo = testo_e.single_mut();
    testo.sections[0].value = energy.0.to_string();

    // Backpack values
    for (mut text, node) in backpack_q.iter_mut() {
        let val = backpack
            .0
            .get(&match_node_content(node))
            .unwrap_or_else(|| &0);
        text.sections[0].value = val.to_string();
    }

    // Goal value
    let mut testo = testo_c.single_mut();
    testo.sections[0].value = coins_to_deposit.deposited.to_string();
}

/// Helper function to translate from a node to a RoboticLibContent
fn match_node_content(node: &ContentNode) -> RoboticLibContent {
    match node {
        ContentNode::RockNode => RoboticLibContent::Rock(0),
        ContentNode::WoodNode => RoboticLibContent::Tree(0),
        ContentNode::FishNode => RoboticLibContent::Fish(0),
        ContentNode::GarbageNode => RoboticLibContent::Garbage(0),
        ContentNode::CoinNode => RoboticLibContent::Coin(0),
    }
}

/// System that lay out the final result with all the transaction of the robot with any content.
fn setup_info_result(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    total_transactions: Res<TotalTransactions>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    // Number of nodes to compute the right height for each one relative to the monitor.
    const NODES: f32 = 12.;
    let mut content_to_push = Vec::new();

    let window_height = window.single().height();
    let box_height = window_height / NODES;

    let container = spawn_container_node![commands, OnResultScreen];
    let sub_container = spawn_sub_container_node![commands, Color::ANTIQUE_WHITE];

    let total_transaction_h = spawn_heading_node![
        commands,
        "Total transactions:",
        50.0,
        box_height,
        JustifyContent::FlexStart
    ];

    content_to_push.extend_from_slice(&[total_transaction_h]);

    for (content_tag, path) in BACKPACK_CONTENT.iter() {
        let val = if let Some(val) = total_transactions
            .0
            .get(&Transaction::new(&match_node_content(content_tag)))
        {
            val.to_string()
        } else {
            0.to_string()
        };
        let content_box =
            spawn_box_node![commands, asset_server, *path, val, *content_tag, box_height];
        content_to_push.push(content_box);
    }

    let quit_button = spawn_button![
        commands,
        asset_server,
        MenuButtonAction::Quit,
        "menu_icons/exit.png",
        "Quit",
        130.
    ];
    content_to_push.push(quit_button);
    commands
        .entity(sub_container)
        .push_children(&content_to_push);
    commands.entity(container).add_child(sub_container);
}

/// Wait for the Terminated event.
fn check_if_simulation_is_terminated(
    mut simulation_data: ResMut<SimulationData>,
    mut menu_state: ResMut<NextState<SimulationState>>,
) {
    if let Some(event) = simulation_data.simulation_events.front() {
        if let MyEvent::RobLib(RoboticLibEvent::Terminated) = event {
            menu_state.set(SimulationState::Result);
            simulation_data.simulation_events.pop_front();
        }
    }
}
