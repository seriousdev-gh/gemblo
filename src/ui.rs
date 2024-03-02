use bevy::prelude::*;

use crate::game::{GameState, player_color};

#[derive(Component)]
struct PlayerText;

pub fn ui_plugin(app: &mut App) {
    app
    .add_systems(Startup, setup)
    .add_systems(Update, print_current_player);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Player: 1",
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
    ));

}

fn print_current_player(
    game: Res<GameState>,
    mut query: Query<&mut Text, With<PlayerText>>
) {
    for mut text in &mut query {
        text.sections[0].style.color = player_color(game.current_player);
        text.sections[0].value = format!("Player: {}", game.current_player + 1);
    }
}
