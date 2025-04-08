// buttons and text color
pub(crate) const TEXT_COLOR: Color = Color::rgb(0., 0., 0.);
pub(crate) const NORMAL_BUTTON: Color = Color::rgb(0.90, 0.90, 0.90);
pub(crate) const HOVERED_BUTTON: Color = Color::rgb(0.75, 0.55, 0.95);
pub(crate) const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
pub(crate) const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
/// Macro used to remove some boiler plate as building an html like user interface in Bevy can be traumatic.
use super::*;
#[macro_export]
// Main container it will hold all the other nodes.
macro_rules! spawn_container_node {
    ($commands:expr) => {
        $commands
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .id()
    };
    ($commands:expr, $tag:expr) => {
        $commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                $tag,
            ))
            .id()
    };
}
#[macro_export]
macro_rules! button_text_style {
    () => {
        TextStyle {
            font_size: 30.0,
            color: TEXT_COLOR,
            ..default()
        }
    };
}
#[macro_export]
macro_rules! spawn_sub_container_node {
    ($commands:expr, $color:expr) => {
        $commands
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: $color.into(),
                ..default()
            })
            .id()
    };
}
#[macro_export]
// Animation button used to choose which bot to use in the simulation.
macro_rules! spawn_animation_button {
    ($commands:expr, $robot_handle:expr, $button_action:expr, $text:expr, $height:expr, $width:expr, $animation_button:expr, $animation_timer:expr) => {
        $commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px($width),
                        height: Val::Px($height),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    border_color: NORMAL_BUTTON.into(),
                    background_color: NORMAL_BUTTON.into(),
                    // background_color: Color::BLACK.into(),
                    ..default()
                },
                $button_action,
                AnimatedButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(20.0),
                            ..default()
                        },
                        image: UiImage::new($robot_handle.0[0].clone()),
                        ..default()
                    },
                    $animation_button,
                    $animation_timer,
                ));
                parent.spawn(TextBundle::from_section(
                    $text,
                    TextStyle {
                        font_size: 25.0,
                        color: TEXT_COLOR,
                        ..default()
                    },
                ));
            })
            .id()
    };
}
#[macro_export]
macro_rules! spawn_button {
    // Menu Button with text and icon.
    ($commands:expr, $asset_server:expr, $button_action:expr, $icon_path:expr, $text:expr, $height:expr) => {
        $commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(600.0),
                        height: Val::Px($height),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
                $button_action,
            ))
            .with_children(|parent| {
                let icon = $asset_server.load($icon_path);
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        // This takes the icons out of the flexbox flow, to be positioned exactly
                        position_type: PositionType::Absolute,
                        // The icon will be close to the left border of the button
                        left: Val::Px(30.0),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                });
                parent.spawn(TextBundle::from_section(
                    $text,
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        ..default()
                    },
                ));
            })
            .id()
    };
    // Commands button only icon.
    ($commands:expr, $asset_server:expr, $button_action:expr, $icon_path:expr, $percent_height: expr) => {
        $commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Percent($percent_height),
                        margin: UiRect::horizontal(Val::Px(10.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    // border_color: Color::BLACK.into(),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                $button_action,
            ))
            .with_children(|button| {
                let icon = $asset_server.load($icon_path);
                button.spawn((ImageBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                },));
            })
            .id()
    };
    // Command button only text.
    ($commands:expr, $button_action:expr, $text:expr, $percent_height: expr) => {
        $commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(500.0),
                        height: Val::Px($percent_height),
                        margin: UiRect::horizontal(Val::Px(10.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    // border_color: Color::BLACK.into(),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                $button_action,
            ))
            .with_children(|button| {
                button.spawn(TextBundle::from_section(
                    $text,
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        ..default()
                    },
                ));
            })
            .id()
    };
}
#[macro_export]
macro_rules! spawn_button_showcase {
    // Menu Button with text and icon.
    ($commands:expr, $asset_server:expr, $button_action:expr, $icon_path:expr, $text:expr, $height:expr) => {
        $commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(600.0),
                        height: Val::Px($height),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
                $button_action,
                ButtonShowcase,
            ))
            .with_children(|parent| {
                let icon = $asset_server.load($icon_path);
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        // This takes the icons out of the flexbox flow, to be positioned exactly
                        position_type: PositionType::Absolute,
                        // The icon will be close to the left border of the button
                        left: Val::Px(30.0),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                });
                parent.spawn(TextBundle::from_section(
                    $text,
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        ..default()
                    },
                ));
            })
            .id()
    };
}
// This will create a node with the following from left to right:
// - icon
// - label
// - value (taken from resource)
// - 2 button to increase/decrease the value
#[macro_export]
macro_rules! spawn_setting_value_node {
    ($commands: expr, $width: expr, $height: expr, $icon_path: expr, $label: expr, $button_text_style: expr, $asset_server: expr, $value:expr, $up_label:expr, $down_label:expr, $action_up:expr, $action_down:expr, $tag:expr) => {
        $commands
            // container
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Px($width),
                    height: Val::Px($height),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            })
            .with_children(|main_box| {
                // icon and label
                main_box
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(60.),
                            height: Val::Percent(100.),
                            margin: UiRect::all(Val::Px(0.0)),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,

                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // icon
                        let icon = $asset_server.load($icon_path);
                        parent.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(60.0),
                                ..default()
                            },
                            image: UiImage::new(icon),
                            ..default()
                        });
                        // label
                        parent.spawn(TextBundle::from_section($label, $button_text_style.clone()));
                    });
                // value
                main_box
                    .spawn(NodeBundle { ..default() })
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                $value.to_string(),
                                $button_text_style.clone(),
                            ),
                            $tag,
                        ));
                    });
                // buttons
                main_box
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(20.0),
                            height: Val::Percent(100.),
                            margin: UiRect::all(Val::Px(0.0)),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(50.),
                                        margin: UiRect::all(Val::Px(0.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(),

                                    ..default()
                                },
                                $action_up,
                            ))
                            .with_children(|up_button| {
                                up_button.spawn(TextBundle::from_section(
                                    $up_label,
                                    $button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(50.0),
                                        margin: UiRect::all(Val::Px(0.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                $action_down,
                            ))
                            .with_children(|up_button| {
                                up_button.spawn(TextBundle::from_section(
                                    $down_label,
                                    $button_text_style.clone(),
                                ));
                            });
                    });
            })
            .id()
    };
}
#[macro_export]
macro_rules! spawn_box_node {
    ($commands:expr, $asset_server:expr, $icon_path:expr, $value:expr, $tag:expr, $box_height:expr) => {
        $commands
            .spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    width: Val::Percent(100.),
                    height: Val::Px($box_height),
                    // border: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                border_color: Color::BLACK.into(),
                ..default()
            })
            .with_children(|parent| {
                let icon = $asset_server.load($icon_path);
                parent.spawn(ImageBundle {
                    style: Style {
                        // justify_content: JustifyContent::Center,
                        left: Val::Px(20.),
                        align_items: AlignItems::Center,
                        width: Val::Px(60.0),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                });
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexStart,
                            width: Val::Px(380.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                $value.to_string(),
                                TextStyle {
                                    font_size: 40.0,
                                    color: TEXT_COLOR,
                                    ..default()
                                },
                            ),
                            $tag,
                        ));
                    });
            })
            .id()
    };
}

#[macro_export]
macro_rules! spawn_heading_node {
    ($commands:expr, $text:expr, $font_size:expr, $height:expr, $justify_c:expr) => {
        $commands
            .spawn(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    left: Val::Px(20.),
                    justify_content: $justify_c,
                    width: Val::Percent(100.),
                    height: Val::Px($height),
                    margin: UiRect::horizontal(Val::Px(50.)),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    $text,
                    TextStyle {
                        font_size: $font_size,
                        color: TEXT_COLOR,
                        ..default()
                    },
                ));
            })
            .id()
    };
}
