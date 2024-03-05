use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::{hex::Hex, CursorWorldCoords};
use crate::game::*;

pub fn pickup_shape(
    world_cursor: Res<CursorWorldCoords>,
    btn: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    hexagons: Query<(&Parent, &GlobalTransform, &PlayerIndex), With<Selectable>>,
    shapes: Query<(&Transform, &Children), (With<HexShape>, Without<Selected>)>,
    selected_shape: Query<Entity, (With<HexShape>, With<Selected>)>,
    mut commands: Commands,
) {
    if !selected_shape.is_empty() || !btn.just_pressed(MouseButton::Left) {
        return;
    }

    for (parent, child_transform, player_index) in hexagons.iter() {
        if game.current_player != player_index.0 {
            continue;
        }

        if hex_collision_with_point(world_cursor.0, child_transform.translation()) {
            let parent_shape_result = shapes.get(parent.get());
            if let Ok(parent_shape) = parent_shape_result {
                game.original_transform = *parent_shape.0;
                game.mouse_offset = parent_shape.0.translation.xy() - world_cursor.0;

                commands.entity(parent.get()).insert(Selected);
                for child in parent_shape.1.iter() {
                    commands.entity(*child).insert(Selected);
                }
            }
            return;
        }
    }
}

pub fn move_shape(
    mut scroll_evr: EventReader<MouseWheel>,
    btn: Res<ButtonInput<KeyCode>>,
    world_cursor: Res<CursorWorldCoords>,
    mut game: ResMut<Game>,
    mut selected_shape: Query<&mut Transform, (With<HexShape>, With<Selected>)>,
    mut selected_hexagons: Query<(&GlobalTransform, &mut Sprite), (With<Selectable>, With<Selected>)>,
) {
    if let Ok(mut shape_transform) = selected_shape.get_single_mut() {
        let mut angle = 0.0_f32;

        if btn.just_pressed(KeyCode::KeyR) {
            if btn.pressed(KeyCode::ShiftLeft) {
                angle = 60.0_f32.to_radians();
            } else {
                angle = -60.0_f32.to_radians();
            }
        }

        for ev in scroll_evr.read() {
            if ev.y > 0.0 {
                angle = 60.0_f32.to_radians();
            } else if ev.y < 0.0 {
                angle = -60.0_f32.to_radians();
            }
        }

        game.mouse_offset = Quat::from_rotation_z(angle).mul_vec3(game.mouse_offset.extend(0.0)).xy();
        shape_transform.rotate(Quat::from_rotation_z(angle));

        shape_transform.translation = Vec3 {
            x: game.mouse_offset.x + world_cursor.0.x,
            y: game.mouse_offset.y + world_cursor.0.y,
            z: SELECTED_Z
        };
    }

    if !selected_hexagons.is_empty() {
        let rounded_shape_hexes: Vec<Hex> = selected_hexagons.iter().map(|(transform, _)|
            pixel_to_hex(transform.translation().xy())
        ).collect();
        let shape_status = action_when_shape_placed(&game.board, &rounded_shape_hexes, game.current_player);
        for (_, mut sprite) in selected_hexagons.iter_mut() {
            let alpha = match shape_status {
                PutShapeAction::ReturnToOrigin => 0.8,
                _ => 1.0
            };
            sprite.color = sprite.color.with_a(alpha);
        }
    }

}

pub fn put_shape(
    btn: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    mut selected_hexagons: Query<(Entity, &GlobalTransform, &mut Sprite), (With<Selectable>, With<Selected>)>,
    mut selected_shape: Query<(Entity, &mut Transform), (With<HexShape>, With<Selected>)>,
    mut commands: Commands,
) {
    if !btn.just_released(MouseButton::Left) {
        return;
    }

    if let Ok((shape_entity, mut shape_transform)) = selected_shape.get_single_mut() {
        let rounded_shape_hexes: Vec<Hex> = selected_hexagons.iter().map(|(_, transform, _)|
            pixel_to_hex(transform.translation().xy())
        ).collect();
        let shape_status = action_when_shape_placed(&game.board, &rounded_shape_hexes, game.current_player);

        match shape_status {
            PutShapeAction::PutOnBoard => {
                let current_player = game.current_player;
                for hex in rounded_shape_hexes {
                    game.board.insert(hex, Cell::Player(current_player));
                }
                for (shape_hex, _, _) in selected_hexagons.iter() {
                    commands.entity(shape_hex).despawn();
                }
                commands.entity(shape_entity).despawn();
                game.current_player = (game.current_player + 1) % game.player_count;


                if let Some(source) = game.drop_audio_handles.choose(&mut rand::thread_rng()) {
                    commands.spawn(AudioBundle {
                        source: source.clone(),
                        settings: PlaybackSettings::DESPAWN
                    });
                }
            },
            PutShapeAction::ReturnToOrigin => {
                shape_transform.translation = game.original_transform.translation;
                shape_transform.rotation = game.original_transform.rotation;
            },
            PutShapeAction::PutOutsideBoard => {
            }
        }

        for (shape_hex, _, mut sprite) in selected_hexagons.iter_mut() {
            sprite.color = sprite.color.with_a(1.0);
            shape_transform.translation.z = DEFAULT_Z;
            commands.entity(shape_hex).remove::<Selected>();
        }
        commands.entity(shape_entity).remove::<Selected>();
    }
}

fn action_when_shape_placed(board: &Board, shape_hexes: &[Hex], current_player: usize) -> PutShapeAction {
    let hexes_on_board = shape_hexes.iter().filter(|hex| board.contains_key(hex)).count();
    if hexes_on_board == shape_hexes.len() {
        if shape_can_be_placed_on_board(board, shape_hexes, current_player) {
            PutShapeAction::PutOnBoard
        } else {
            PutShapeAction::ReturnToOrigin
        }
    } else if hexes_on_board > 0 && hexes_on_board < shape_hexes.len() {
        PutShapeAction::ReturnToOrigin
    } else {
        PutShapeAction::PutOutsideBoard
    }
}

fn shape_can_be_placed_on_board(board: &Board, shape_hexes: &[Hex], current_player: usize) -> bool {
    for hex in shape_hexes {
        match board.get(hex) {
            // always can place on own starting square
            Some(Cell::PlayerStart(index)) if *index == current_player => return true,
            // can't place when have cell occupied
            Some(Cell::Player(_)) => return false,
            Some(Cell::Disabled) => return false,
            _ => ()
        }

        // can't place when have direct neighbours
        if NEIGHBOURS.iter().any(|n|
            is_hex_belong_to_player(board, hex.add(n), current_player)
        ) {
            return false;
        }
    }

    shape_hexes.iter().any(|hex|
        DIAGONAL_NEIGHBOURS.iter().any(|(diagonal, near_1, near_2)|
            is_hex_belong_to_player(board, hex.add(diagonal), current_player) &&
            is_hexes_belong_to_different_players(board, hex.add(near_1), hex.add(near_2))
        )
    )
}


fn is_hex_belong_to_player(board: &Board, hex: Hex, player_index: usize) -> bool {
    matches!(board.get(&hex), Some(Cell::Player(i)) if *i == player_index)
}

fn is_hexes_belong_to_different_players(board: &Board, hex1: Hex, hex2: Hex) -> bool {
    match (board.get(&hex1), board.get(&hex2)) {
        (Some(Cell::Player(i)), Some(Cell::Player(j))) => i != j,
        _ => true
    }
}

pub fn board_system(
    mut board_hexes: Query<(&mut Sprite, &Hex), With<BoardHex>>,
    game: Res<Game>,
    selected_hexagons: Query<&GlobalTransform, (With<Selectable>, With<Selected>)>,
) {
    let selected_hexes: Vec<Hex> = selected_hexagons.iter().map(|transform|
        pixel_to_hex(transform.translation().xy())
    ).collect();

    for (mut sprite, hex) in &mut board_hexes {
        if selected_hexes.contains(hex) {
            let action = action_when_shape_placed(&game.board, &selected_hexes, game.current_player);
            if matches!(action, PutShapeAction::PutOnBoard) {
                sprite.color = Color::GRAY;
                continue;
            }
        }

        if let Some(cell) = game.board.get(hex) {
            match cell {
                Cell::Empty => sprite.color = Color::WHITE,
                Cell::Player(i) => {
                    sprite.color = player_color(*i);
                }
                Cell::PlayerStart(i) => {
                    sprite.color = player_color_darken(*i);
                }
                Cell::Disabled => sprite.color = Color::DARK_GRAY
            };
        }
    }
}

fn hex_collision_with_point(point: Vec2, translation: Vec3) -> bool{
    translation.xy().distance_squared(point) <= HEX_RADIUS * HEX_RADIUS
}
