use bevy::prelude::*;
use crate::hex::{Hex, Rotation};

use Rotation::*;
use crate::game::*;

const BOARD_SECTOR: [i32; 11] = [0, 11, 10, 10, 9, 9, 8, 8, 6, 4, 2];
const BOARD_SECTOR_SMALL: [i32; 8] = [0, 8, 7, 7, 6, 6, 4, 2];

pub fn call(
    mut commands: Commands,
    mut game: ResMut<Game>,
    asset_server: Res<AssetServer>
) {
    let hex_texture_handle = &asset_server.load("hex.png");

    game.drop_audio_handles = vec![
        asset_server.load("drop1.ogg"),
        asset_server.load("drop2.ogg"),
        asset_server.load("drop3.ogg"),
        asset_server.load("drop4.ogg"),
        asset_server.load("drop5.ogg")
    ];

    fill_board(&mut game.board, true, false);
    let player_count = game.player_count;
    disable_unused(&mut game.board, player_count);

    commands.spawn((OnGameScreen, BoardComponent, SpatialBundle::default())).with_children(|parent| {
        for (hex, _) in game.board.iter() {
            let location = hex_to_pixel(hex);
            parent.spawn(
                (
                    build_hex(location, hex_texture_handle, Color::WHITE),
                    BoardHex,
                    *hex
                )
            );
        }
    });

    spawn_pieces(&mut commands, hex_texture_handle, 0, Vec3 { x: 10.0 * HEX_WIDTH, y: -7.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, hex_texture_handle, 1, Vec3 { x: -20.0 * HEX_WIDTH, y: -7.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, hex_texture_handle, 2, Vec3 { x: -25.0 * HEX_WIDTH, y: 4.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, hex_texture_handle, 3, Vec3 { x: -20.0 * HEX_WIDTH, y: 15.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, hex_texture_handle, 4, Vec3 { x: 10.0 * HEX_WIDTH, y: 15.0 * HEX_WIDTH, z: 0.0  });
    spawn_pieces(&mut commands, hex_texture_handle, 5, Vec3 { x: 15.0 * HEX_WIDTH, y: 4.0 * HEX_WIDTH, z: 0.0  });
}

fn spawn_pieces(commands: &mut Commands, texture: &Handle<Image>, player_index: usize, starting_translation: Vec3) {
    for piece_hexes in ALL_PIECES {
        spawn_piece(commands, texture, player_index, piece_hexes, starting_translation);
    }
}

fn spawn_piece(commands: &mut Commands, texture: &Handle<Image>, player_index: usize, hexes: &[(i32, i32)], starting_translation: Vec3) {
    let base = Hex { q: hexes[0].0, r: hexes[0].1 };
    let translation = starting_translation + hex_to_pixel(&base).extend(0.0);

    commands.spawn((
        OnGameScreen,
        HexShape,
        PlayerIndex(player_index),
        SpatialBundle { transform: Transform::from_translation(translation), ..default() }
    )).with_children(|parent| {
        for tuple in hexes {
            let hex = Hex { q: tuple.0, r: tuple.1 };
            let relative_translation = hex_to_pixel(&hex.sub(&base));
            parent.spawn((
                build_hex(relative_translation, texture, player_color(player_index)),
                Selectable,
                PlayerIndex(player_index)
            ));
        }
    });
}

fn build_hex(location: Vec2, texture: &Handle<Image>, color: Color) -> SpriteBundle {
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

fn fill_board(board: &mut Board, disabled: bool, small: bool) {
    board.insert(Hex { q: 0, r: 0 }, Cell::Empty);
    for rotation in [Rot0,Rot60Cw,Rot120Cw,Rot180,Rot60Ccw,Rot120Ccw] {
        if small {
            fill_board_sector_small(board, rotation);
        } else {

            fill_board_sector(board, rotation, disabled);
        }
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
            board.insert(Hex { q: q as i32, r }.rotate(rotation), cell);
        }
    }
}

fn fill_board_sector_small(board: &mut Board, rotation: Rotation) {
    for q in 0..8 {
        for r in 0..BOARD_SECTOR_SMALL[q as usize] {
            board.insert(Hex { q: q as i32, r }.rotate(rotation), Cell::Empty);
        }
    }
}

fn disable_unused(board: &mut Board, player_count: usize) {
    match player_count {
        2 | 4 => {
            two_player_setup(board)
        }
        3 => {
            three_player_setup(board)
        }
        6 => {
            six_player_setup(board)
        }
        _ => todo!()
    }
}

fn two_player_setup(board: &mut Board) {
    todo!()
}

fn three_player_setup(board: &mut Board) {
    fill_board(board, false, true);

    board.insert(Hex { q: 5, r: 5 }, Cell::PlayerStart(0));
    board.insert(Hex { q: 5, r: 5 }.rotate(Rot120Cw), Cell::PlayerStart(1));
    board.insert(Hex { q: 5, r: 5 }.rotate(Rot120Ccw), Cell::PlayerStart(2));
}

fn six_player_setup(board: &mut Board) {
    fill_board(board, false, false);

    board.insert(Hex { q: 7, r: 7 }, Cell::PlayerStart(0));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot60Cw), Cell::PlayerStart(1));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot120Cw), Cell::PlayerStart(2));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot180), Cell::PlayerStart(3));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot120Ccw), Cell::PlayerStart(4));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot60Ccw), Cell::PlayerStart(5));
}