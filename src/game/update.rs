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
                game.pass_turn_count = 0;
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
    let outside_board = shape_hexes.iter().all(|hex| !board.contains_key(hex));

    if outside_board {
        PutShapeAction::PutOutsideBoard
    } else if shape_can_be_placed_on_board(board, shape_hexes, current_player) {
        PutShapeAction::PutOnBoard
    } else {
        PutShapeAction::ReturnToOrigin
    }
}

fn shape_can_be_placed_on_board(board: &Board, shape_hexes: &[Hex], current_player: usize) -> bool {
    for hex in shape_hexes {
        match board.get(hex) {
            // always can place on own starting square
            Some(Cell::PlayerStart(index)) if *index == current_player => return true,
            // can't place when cell occupied
            Some(Cell::Player(_) | Cell::Disabled) => return false,
            // can't place when partially on board
            None => return false,
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
        if selected_hexes.contains(hex) && shape_can_be_placed_on_board(&game.board, &selected_hexes, game.current_player) {
            sprite.color = Color::GRAY;
            continue;
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

pub fn on_pass_turn(
    mut ev_pass: EventReader<PassTurnEvent>,
    mut game: ResMut<Game>,
    mut game_state: ResMut<NextState<GameState>>,
    blocks: Query<(&Selectable, &PlayerIndex)>,
    pieces: Query<(&HexShape, &PlayerIndex)>,
) {
    for _ev in ev_pass.read() {
        game.current_player = (game.current_player + 1) % game.player_count;
        game.pass_turn_count += 1;
        let players_have_turns = game.pass_turn_count < game.player_count;
        if players_have_turns {
            continue
        }

        game_state.set(GameState::GameEnd);

        let players_stats: Vec<PlayerStats> = players_stats(game.player_count, &blocks, &pieces);

        game.winner_player = detect_winner(players_stats);

        if let Some(index) = game.winner_player {
            println!("Winner is {index:?}");
        } else {
            println!("No winner");
        }

    }
}

fn players_stats(
    player_count: usize,
    blocks: &Query<(&Selectable, &PlayerIndex)>,
    pieces: &Query<(&HexShape, &PlayerIndex)>
) -> Vec<PlayerStats> {
    let mut players_stats: Vec<PlayerStats> = Vec::new();
    for i in 0..player_count {
        let blocks_count =
            blocks
                .iter()
                .filter(|(_, &PlayerIndex(player_index))| player_index == i)
                .count();

        let smallest_piece_option =
            pieces
                .iter()
                .filter(|(_, &PlayerIndex(player_index))| player_index == i)
                .map(|(&HexShape(size), _)| size)
                .max();

        if let Some(smallest_piece) = smallest_piece_option {
            players_stats.push(PlayerStats { index: i, blocks: blocks_count, largest_piece: smallest_piece });
        }
    }
    players_stats
}

#[derive(Debug)]
struct PlayerStats {
    index: usize,
    blocks: usize,
    largest_piece: usize
}

fn hex_collision_with_point(point: Vec2, translation: Vec3) -> bool{
    translation.xy().distance_squared(point) <= HEX_RADIUS * HEX_RADIUS
}

fn detect_winner(
    mut players_stats: Vec<PlayerStats>,
) -> Option<usize> {
    println!("Players stats: {players_stats:?}");
    if players_stats.is_empty() {
        println!("No players with pieces");
        return None;
    }

    // rule 1
    let minimum_blocks = players_stats.iter().map(|s| s.blocks).min().unwrap();
    players_stats.retain(|s| s.blocks == minimum_blocks);
    println!("Players stats with minimum number of blocks: {players_stats:?}");
    if players_stats.len() == 1 {
        let player_stat = players_stats.first().unwrap();
        return Some(player_stat.index);
    }

    // rule 2
    let smallest_largest_piece = players_stats.iter().map(|s| s.largest_piece).min().unwrap();
    players_stats.retain(|s| s.largest_piece == smallest_largest_piece);
    println!("Players stats with smallest largest piece: {players_stats:?}");
    if players_stats.len() == 1 {
        let player_stat = players_stats.first().unwrap();
        return Some(player_stat.index);
    }

    todo!("rule 3 not implemented");
}

// fn is_game_end(player_index: usize, board: &Board) {
//     // If no player can make a move, the game ends.
//     // If a player has no pieces or no valid moves, the player cannot make a move.
//     // The player with fewer blocks comprising their pieces wins,
//     // otherwise the player with the largest piece smaller than the rest of the players wins.
//     // Otherwise, the algorithm for calculating the value of pieces is used.
//     let player_placed_blocks: Vec<Hex> =
//         board.iter().filter_map(|(&k, v)|
//             if matches!(v, Cell::Player(i) if *i == player_index) {
//                 Some(k)
//             } else {
//                 None
//             }
//         ).collect();

// }

// // piece_blocks - first block is always Hex { 0, 0 }
// //
// // this function is not optimized
// fn is_piece_fits(placed_blocks: &[Hex], piece_blocks: &[Hex], board: &Board, current_player: usize) -> bool {
//     //  find already placed cells
//     //      find this cell neigbours
//     //          for each piece rotation
//     //              for each starting block
//     //                  return true if piece fits

//     for &placed_block in placed_blocks {
//         for (offset, _, _) in DIAGONAL_NEIGHBOURS {
//             let block_to_check = placed_block + offset;

//             match board.get(&block_to_check) {
//                 Some(Cell::Empty | Cell::PlayerStart(_)) => (),
//                 _ => continue
//             }

//             for rotation in ALL_ROTATIONS {
//                 assert_eq!(piece_blocks[0], Hex::ZERO);
//                 let rotated_blocks: Vec<Hex> = piece_blocks.iter().map(|block| block.rotate(rotation)).collect();
//                 assert_eq!(rotated_blocks[0], Hex::ZERO);

//                 for starting_block in rotated_blocks.as_slice() {
//                     let blocks_with_offset: Vec<Hex> =
//                         rotated_blocks.iter().map(|block| *block - *starting_block).collect();

//                     if shape_can_be_placed_on_board(board, &blocks_with_offset, current_player) {
//                         return true;
//                     }
//                 }
//             }
//         }
//     }

//     false
// }
