use std::f32::consts::SQRT_3;

use bevy::prelude::*;
use std::collections::HashMap;

use crate::GameState;
use crate::despawn_screen;
use crate::hex::Hex;
use crate::game::update::*;

mod setup;
mod update;

pub fn game_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Game), setup::call)
        .add_systems(Update, (pickup_shape, board_system).run_if(in_state(GameState::Game)))
        .add_systems(Update, (move_shape, put_shape).chain().run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
}

#[derive(Resource, Default)]
pub struct Game {
    original_transform: Transform,
    mouse_offset: Vec2,
    board: Board,
    player_count: usize,
    pub current_player: usize,
    drop_audio_handles: Vec<Handle<AudioSource>>
}

impl Game {
    pub fn new(player_count: usize) -> Self {
        Self {
            player_count,
            ..default()
        }
    }
}

#[derive(Component)]
struct BoardComponent;

#[derive(Component)]
struct BoardHex;

#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct PlayerIndex(usize);

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct HexShape;

#[derive(Component)]
struct OnGameScreen;

#[derive(PartialEq, Default, Debug)]
enum Cell {
    #[default]
    Empty,
    Disabled,
    Player(usize),
    PlayerStart(usize)
}

enum PutShapeAction {
    PutOnBoard,
    ReturnToOrigin,
    PutOutsideBoard
}

const DEFAULT_Z: f32 = 1.0;
const SELECTED_Z: f32 = 1.0001;

const NEIGHBOURS: [Hex; 6] = [Hex { q:  0, r: 1 }, Hex { q: 1, r:   0 }, Hex { q:  0, r: -1 },
                              Hex { q: -1, r: 0 }, Hex { q: 1, r:  -1 }, Hex { q: -1, r:  1 }];
const DIAGONAL_NEIGHBOURS: [(Hex, Hex, Hex); 6] = [
    (Hex { q:  1, r:  1 }, Hex { q:  1, r:  0 }, Hex { q:  0, r:  1 }),
    (Hex { q: -1, r:  2 }, Hex { q:  0, r:  1 }, Hex { q: -1, r:  1 }),
    (Hex { q: -2, r:  1 }, Hex { q: -1, r:  1 }, Hex { q: -1, r:  0 }),
    (Hex { q: -1, r: -1 }, Hex { q: -1, r:  0 }, Hex { q:  0, r: -1 }),
    (Hex { q:  1, r: -2 }, Hex { q:  0, r: -1 }, Hex { q:  1, r: -1 }),
    (Hex { q:  2, r: -1 }, Hex { q:  1, r: -1 }, Hex { q:  1, r:  0 })
];

const MAX_PLAYERS: usize = 6;
const HEX_SCALE: f32 = 0.25;
const HEX_REAL_WIDTH_IN_PIXELS: f32 = 128.0;
const HEX_WIDTH: f32 = HEX_REAL_WIDTH_IN_PIXELS * HEX_SCALE;
const HEX_RADIUS: f32 = HEX_WIDTH / 2.0;

const ALL_PIECES: [&[(i32, i32)]; 18] =
[
    // 8 - 5 hexagons
    &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
    &[(2, -1), (2, 0), (3, 0), (4, 0), (4, 1)],
    &[(4, -2), (5, -2), (6, -2), (6, -1), (6, 0)],
    &[(8, -4), (9, -4), (10, -5), (9, -3), (8, -2)],
    &[(12, -6), (13, -6), (13, -5), (14, -5), (14, -4)],
    &[(2, 2), (2, 3), (2, 4), (1, 5), (3, 4)],
    &[(5, 2), (6, 2), (7, 1), (8, 1), (9, 1)],
    &[(9, -1), (10, -2), (10, -1), (11, -3), (11, -1)],

    // 5 - 4  hexagons
    &[(0, 7), (0, 8), (0, 9), (0, 10)],
    &[(2, 6), (2, 7), (3, 7), (3, 8)],
    &[(4, 5), (5, 4), (5, 5), (6, 4)],
    &[(5, 7), (6, 6), (7, 6), (7, 7)],
    &[(13, -2), (13, -1), (14, -1), (12, 0)],

    // 3 - 3 hexagons
    &[(8, 3), (9, 3), (8, 4)],
    &[(11, 2), (11, 3), (10, 4)],
    &[(14, 1), (14, 2), (14, 3)],

    &[(11, 5), (12, 4)],
    &[(9, 6 )],
];

type Board = HashMap<Hex, Cell>;

pub fn player_color(player_index: usize) -> Color {
    Color::hsl(player_index as f32 / MAX_PLAYERS as f32 * 360.0, 1.0, 0.5)
}

fn hex_to_pixel(hex: &Hex) -> Vec2 {
    Vec2 {
        x: HEX_RADIUS * (3./2. * hex.q as f32),
        y: -HEX_RADIUS * (SQRT_3/2. * hex.q as f32  + SQRT_3 * hex.r as f32)
    }
}

fn pixel_to_hex(pixel: Vec2) -> Hex {
    let q = ( 2./3.0 * pixel.x) / HEX_RADIUS;
    let r = (-1./3.0 * pixel.x + SQRT_3/3.0 * -pixel.y) / HEX_RADIUS;
    Hex::from_fraction(q, r)
}

fn player_color_darken(player_index: usize) -> Color {
    Color::hsl(player_index as f32 / MAX_PLAYERS as f32 * 360.0, 0.9, 0.4)
}

