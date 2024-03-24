use std::f32::consts::SQRT_3;

use bevy::prelude::*;
use std::collections::HashMap;

use crate::hex::Rotation;
use crate::GameState;
use crate::despawn_screen;
use crate::hex::Hex;
use crate::game::update::*;
use Rotation::*;

mod setup;
mod update;

pub fn game_plugin(app: &mut App) {
    app
        .add_event::<PassTurnEvent>()
        .add_systems(OnEnter(GameState::Game), (despawn_screen::<OnGameScreen>, setup::call).chain())
        .add_systems(Update, (pickup_piece, board_system, on_pass_turn).run_if(in_state(GameState::Game)))
        .add_systems(Update, (move_piece, put_piece).chain().run_if(in_state(GameState::Game)))
        .add_systems(OnEnter(GameState::Menu), despawn_screen::<OnGameScreen>)
        .add_systems(OnExit(GameState::GameEnd), despawn_screen::<OnGameScreen>);
}

#[derive(Resource, Default)]
pub struct Game {
    original_transform: Transform,
    mouse_offset: Vec2,
    board: Board,
    player_count: usize,
    pub current_player: usize,
    pub winner_player: Option<usize>,
    drop_audio_handles: Vec<Handle<AudioSource>>,
    pass_turn_count: usize,
}

impl Game {
    pub fn new(player_count: usize) -> Self {
        Self {
            player_count,
            ..default()
        }
    }
}

type NumberOfBlocks = usize;

#[derive(Event)]
pub struct PassTurnEvent;

#[derive(Component)]
struct BoardComponent;

#[derive(Component)]
struct BoardHex;

#[derive(Component)]
struct BlockSelectable;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct PlayerIndex(usize);

#[derive(Component)]
struct Piece(NumberOfBlocks);

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

enum PutPieceAction {
    PutOnBoard,
    ReturnToOrigin,
    PutOutsideBoard
}

const ALL_ROTATIONS: [Rotation; 6] = [Rot0, Rot60Cw, Rot120Cw, Rot180, Rot60Ccw, Rot120Ccw];
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

