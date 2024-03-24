use bevy::prelude::*;

use crate::{
    despawn_screen,
    game::{player_color, Game, PassTurnEvent},
    GameState,
};

pub fn ui_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), setup)
        .add_systems(OnEnter(GameState::GameEnd), setup_for_game_end)
        .add_systems(
            Update,
            (
                button_system,
                button_action,
                (print_current_player).run_if(in_state(GameState::Game)),
                (print_winner_info).run_if(in_state(GameState::GameEnd)),
            )
            ,
        )
        .add_systems(OnEnter(GameState::Menu), despawn_screen::<OnUiScreen>)
        .add_systems(OnExit(GameState::GameEnd), despawn_screen::<OnUiScreen>);
}

#[derive(Component)]
enum UiButtonAction {
    Menu,
    Pass,
}

#[derive(Component)]
struct PlayerText;

#[derive(Component)]
struct OnUiScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(75.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                padding: UiRect { left: Val::Px(10.0), right: Val::Px(10.0), top: Val::Px(10.0), bottom: Val::Px(10.0) },
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(OnUiScreen)
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    UiButtonAction::Menu,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Menu",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    ));
                });

            // game information
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 25.0,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    ..default()
                }),
                PlayerText,
                OnUiScreen,
            ));

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    UiButtonAction::Pass,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Give up",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    ));
                });
        });
}

fn setup_for_game_end(to_despawn: Query<(Entity, &UiButtonAction)>, mut commands: Commands){
    for (entity, uid_button_action) in &to_despawn {
        if matches!(uid_button_action, UiButtonAction::Pass){
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn button_action(
    interaction_query: Query<(&Interaction, &UiButtonAction), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut ev_pass: EventWriter<PassTurnEvent>,
) {
    for (interaction, ui_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match ui_button_action {
                UiButtonAction::Menu => game_state.set(GameState::Menu),
                UiButtonAction::Pass => {
                    ev_pass.send(PassTurnEvent);
                }
            }
        }
    }
}

fn print_current_player(game: Res<Game>, mut query: Query<&mut Text, With<PlayerText>>) {
    for mut text in &mut query {
        text.sections[0].style.color = player_color(game.current_player);
        text.sections[0].value = format!("Player: {}", game.current_player + 1);
    }
}

fn print_winner_info(game: Res<Game>, mut query: Query<&mut Text, With<PlayerText>>) {
    for mut text in &mut query {
        text.sections[0].style.color = Color::WHITE;
        if let Some(index) = game.winner_player {
            text.sections[0].value = format!("Player: {} is winner", index + 1);
        } else {
            text.sections[0].value = "Draw".to_string();
        }

    }
}
