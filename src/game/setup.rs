use bevy::prelude::*;
use crate::hex::{Hex, Rotation};

use crate::game::*;

// index is q-axis, value is length along r-axis
const BOARD_SECTOR: [i32; 11] = [0, 11, 10, 10, 9, 9, 8, 8, 6, 4, 2];
const BOARD_SECTOR_SMALL: [i32; 8] = [0, 8, 7, 7, 6, 6, 4, 2];

const ALL_PIECES: [&[(i32, i32)]; 11] =
[
    // &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8), (0, 9), (0, 10), (0, 11), (0, 12), (0, 13), (0, 14)],
    // &[(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8), (2, 9), (2, 10), (2, 11), (2, 12), (2, 13), (2, 14)],
    // &[(4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6), (4, 7), (4, 8), (4, 9), (4, 10), (4, 11), (4, 12), (4, 13), (4, 14)],

    // 8 - 5 hexagons
    &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
    // &[(2, -1), (2, 0), (3, 0), (4, 0), (4, 1)],
    // &[(4, -2), (5, -2), (6, -2), (6, -1), (6, 0)],
    // &[(8, -4), (9, -4), (10, -5), (9, -3), (8, -2)],
    // &[(12, -6), (13, -6), (13, -5), (14, -5), (14, -4)],
    // &[(2, 2), (2, 3), (2, 4), (1, 5), (3, 4)],
    // &[(5, 2), (6, 2), (7, 1), (8, 1), (9, 1)],
    // &[(9, -1), (10, -2), (10, -1), (11, -3), (11, -1)],

    // 5 - 4  hexagons
    &[(0, 7), (0, 8), (0, 9), (0, 10)],
    &[(2, 6), (2, 7), (3, 7), (3, 8)],
    &[(4, 5), (5, 4), (5, 5), (6, 4)],
    &[(5, 7), (6, 6), (7, 6), (7, 7)],
    &[(13, -2), (13, -1), (14, -1), (12, 0)],

    // // 3 - 3 hexagons
    &[(8, 3), (9, 3), (8, 4)],
    &[(11, 2), (11, 3), (10, 4)],
    &[(14, 1), (14, 2), (14, 3)],

    &[(11, 5), (12, 4)],
    &[(9, 6 )],
];

pub fn call(
    mut commands: Commands,
    mut game: ResMut<Game>,
    asset_server: Res<AssetServer>
) {
    let block_texture_handle = &asset_server.load("hex.png");

    game.drop_audio_handles = vec![
        asset_server.load("drop1.ogg"),
        asset_server.load("drop2.ogg"),
        asset_server.load("drop3.ogg"),
        asset_server.load("drop4.ogg"),
        asset_server.load("drop5.ogg")
    ];

    let player_count = game.player_count;

    fill_board(&mut game.board, true);
    setup_board_for_players(&mut game.board, player_count);

    commands.spawn((OnGameScreen, BoardComponent, SpatialBundle::default())).with_children(|parent| {
        for &hex in game.board.keys() {
            parent.spawn(
                (
                    build_block_sprite(hex, block_texture_handle, Color::WHITE),
                    BoardHex,
                    hex
                )
            );
        }
    });

    let piece_sets_count = if player_count == 2 { 4 } else { player_count };

    spawn_pieces(&mut commands, block_texture_handle, 0, Vec3 { x: 10.0 * HEX_WIDTH, y: -7.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, block_texture_handle, 1, Vec3 { x: -20.0 * HEX_WIDTH, y: -7.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, block_texture_handle, 2, Vec3 { x: -25.0 * HEX_WIDTH, y: 4.0 * HEX_WIDTH, z: 0.0  });
    if piece_sets_count > 3 {
        spawn_pieces(&mut commands, block_texture_handle, 3, Vec3 { x: -20.0 * HEX_WIDTH, y: 15.0 * HEX_WIDTH, z: 0.0  });
    }
    if piece_sets_count > 4 {
        spawn_pieces(&mut commands, block_texture_handle, 4, Vec3 { x: 10.0 * HEX_WIDTH, y: 15.0 * HEX_WIDTH, z: 0.0  });
        spawn_pieces(&mut commands, block_texture_handle, 5, Vec3 { x: 15.0 * HEX_WIDTH, y: 4.0 * HEX_WIDTH, z: 0.0  });
    }
}

fn spawn_pieces(commands: &mut Commands, texture: &Handle<Image>, player_index: usize, starting_translation: Vec3) {
    for piece_blocks in ALL_PIECES {
        spawn_piece(commands, texture, player_index, piece_blocks, starting_translation);
    }
}

fn spawn_piece(commands: &mut Commands, texture: &Handle<Image>, player_index: usize, blocks: &[(i32, i32)], starting_translation: Vec3) {
    let base = Hex { q: blocks[0].0, r: blocks[0].1 };
    let translation = starting_translation + hex_to_pixel(&base).extend(0.0);

    commands.spawn((
        OnGameScreen,
        Piece(blocks.len()),
        PlayerIndex(player_index),
        SpatialBundle { transform: Transform::from_translation(translation), ..default() }
    )).with_children(|parent| {
        for tuple in blocks {
            let hex = Hex { q: tuple.0, r: tuple.1 };
            let relative_translation = hex - base;
            parent.spawn((
                build_block_sprite(relative_translation, texture, player_color(player_index)),
                BlockSelectable,
                PlayerIndex(player_index)
            ));
        }
    });
}

fn build_block_sprite(hex: Hex, texture: &Handle<Image>, color: Color) -> SpriteBundle {
    let location = hex_to_pixel(&hex);
    SpriteBundle {
        sprite: Sprite {
            color,
            ..default()
        },
        texture: texture.clone(),
        transform: Transform::from_xyz(location.x, location.y, 0.).with_scale(Vec3 {x: HEX_SCALE, y: HEX_SCALE, z: 1.0 }),
        ..default()
    }
}

fn fill_board(board: &mut Board, disabled: bool) {
    board.insert(Hex { q: 0, r: 0 }, Cell::Empty);
    for rotation in ALL_ROTATIONS {
        fill_board_sector(board, rotation, disabled);
    }
}

fn fill_board_sector(board: &mut Board, rotation: Rotation, disabled: bool) {
    for q in 0..11 {
        for r in 0..BOARD_SECTOR[q as usize] {
            let cell = if disabled {
                Cell::Disabled
            } else {
                Cell::Empty
            };
            board.insert(Hex { q, r }.rotate(rotation), cell);
        }
    }
}

fn fill_board_sector_small(board: &mut Board, rotation: Rotation) {
    for q in 0..8 {
        for r in 0..BOARD_SECTOR_SMALL[q as usize] {
            board.insert(Hex { q, r }.rotate(rotation), Cell::Empty);
        }
    }
}

fn setup_board_for_players(board: &mut Board, player_count: usize) {
    match player_count {
        2 | 4 => {
            four_player_setup(board);
        }
        3 => {
            three_player_setup(board);
        }
        5 | 6 => {
            six_player_setup(board);
        }
        _ => panic!("not implemented")
    }
}

fn four_player_setup(board: &mut Board) {
    for q in -9..=9 {
        let r_length = (((q + 9) as f32) / 2.0).floor() as i32 + 4;
        for r in 0..r_length {
            board.insert(Hex { q, r: -r }, Cell::Empty);
            board.insert(Hex { q: -q, r }, Cell::Empty);
        }
    }

    board.insert(Hex { q: 9, r: 3 }, Cell::PlayerStart(0));
    board.insert(Hex { q: -9, r: 12 }, Cell::PlayerStart(1));
    board.insert(Hex { q: -9, r: -3 }, Cell::PlayerStart(2));
    board.insert(Hex { q: 9, r: -12 }, Cell::PlayerStart(3));
}

fn three_player_setup(board: &mut Board) {
    for rotation in ALL_ROTATIONS {
        fill_board_sector_small(board, rotation);
    }

    board.insert(Hex { q: 5, r: 5 }, Cell::PlayerStart(0));
    board.insert(Hex { q: 5, r: 5 }.rotate(Rot120Cw), Cell::PlayerStart(1));
    board.insert(Hex { q: 5, r: 5 }.rotate(Rot120Ccw), Cell::PlayerStart(2));
}

fn six_player_setup(board: &mut Board) {
    fill_board(board, false);

    board.insert(Hex { q: 7, r: 7 }, Cell::PlayerStart(0));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot60Cw), Cell::PlayerStart(1));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot120Cw), Cell::PlayerStart(2));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot180), Cell::PlayerStart(3));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot120Ccw), Cell::PlayerStart(4));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot60Ccw), Cell::PlayerStart(5));
}