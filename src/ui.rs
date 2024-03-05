use bevy::prelude::*;

use crate::{despawn_screen, game::{player_color, Game}, GameState};

pub fn ui_plugin(app: &mut App) {
    app
    .add_systems(OnEnter(GameState::Game), (setup, button_setup))
    .add_systems(Update, (print_current_player, button_system).run_if(in_state(GameState::Game)))
    .add_systems(OnExit(GameState::Game), despawn_screen::<OnUiScreen>);
}

#[derive(Component)]
struct MenuButton;

#[derive(Component)]
struct PlayerText;

#[derive(Component)]
struct OnUiScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Player:",
            TextStyle {
                font_size: 25.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        PlayerText,
        OnUiScreen,
    ));
}

fn button_setup(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            OnUiScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    MenuButton
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Menu",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    ));
                });
        });
}

fn print_current_player(
    game: Res<Game>,
    mut query: Query<&mut Text, With<PlayerText>>
) {
    for mut text in &mut query {
        text.sections[0].style.color = player_color(game.current_player);
        text.sections[0].value = format!("Player: {}", game.current_player + 1);
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor
        ),
        (Changed<Interaction>, With<Button>, With<MenuButton>),
    >,
    mut game_state: ResMut<NextState<GameState>>
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                game_state.set(GameState::Menu);
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
