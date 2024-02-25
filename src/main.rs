#![feature(more_float_constants)]

mod hex;
use std::f32::consts::SQRT_3;

use bevy::input::mouse::MouseWheel;
use bevy::render::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use hex::Hex;
use std::collections::HashMap;

#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

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

const DEFAULT_Z: f32 = 1.0;
const SELECTED_Z: f32 = 1.0001;

type Board = HashMap<Hex, Cell>;

const NEIGHBOURS: [Hex; 6] = [Hex { q:  0, r: 1 }, Hex { q: 1, r:   0 }, Hex { q:  0, r:  -1 },
                              Hex { q: -1, r: 0 }, Hex { q:  1, r: -1 }, Hex { q:  -1, r:  1 }];
const DIAGONAL_NEIGHBOURS: [Hex; 6] = [Hex { q:  1, r: 1 }, Hex { q: -1, r:  2 }, Hex { q: -2, r:  1 },
                                       Hex { q: -1, r: 1 }, Hex { q:  1, r: -2 }, Hex { q:  2, r: -1 }];

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

const MAX_PLAYERS: usize = 6;
const HEX_SCALE: f32 = 0.25;
const HEX_REAL_WIDTH_IN_PIXELS: f32 = 128.0;
const HEX_WIDTH: f32 = HEX_REAL_WIDTH_IN_PIXELS * HEX_SCALE;
const HEX_RADIUS: f32 = HEX_WIDTH / 2.0;

#[derive(PartialEq, Default, Debug)]
enum Cell {
    #[default]
    Empty,
    Player(usize),
    PlayerStart(usize)
}

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
// -2 1
#[derive(Resource, Default)]
struct Game {
    original_transform: Transform,
    mouse_offset: Vec2,
    board: Board,
    player_count: usize,
    current_player: usize
}

const BOARD_SECTOR: [i32; 10] = [11, 10, 10, 9, 9, 8, 8, 6, 4, 2];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
            primary_window: Some(Window {
                position: WindowPosition::At(IVec2 { x: 0, y: 0 }),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Game { player_count: 2, ..default() })
        .insert_resource(CursorWorldCoords { ..default() })
        .add_systems(Startup, setup)
        .add_systems(Update, (world_cursor_system, pickup_shape, board_system))
        .add_systems(Update, (move_shape, put_shape).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut game: ResMut<Game>,
    asset_server: Res<AssetServer>
) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::AutoMin { min_width: 1920.0, min_height: 1080.0 };
    commands.spawn(camera_bundle);

    let hex_texture_handle = &asset_server.load("hex.png");

    fill_board(&mut game.board);

    commands.spawn((BoardComponent, SpatialBundle::default())).with_children(|parent| {
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

fn player_color(player_index: usize) -> Color {
    Color::hsl(player_index as f32 / MAX_PLAYERS as f32 * 360.0, 1.0, 0.5)
}

fn player_color_darken(player_index: usize) -> Color {
    Color::hsl(player_index as f32 / MAX_PLAYERS as f32 * 360.0, 0.9, 0.4)
}

fn fill_board(board: &mut Board) {
    use hex::Rotation::*;
    board.insert(Hex { q: 0, r: 0 }, Cell::Empty);
    fill_board_sector(board, Rot0);
    fill_board_sector(board, Rot60Cw);
    fill_board_sector(board, Rot120Cw);
    fill_board_sector(board, Rot180);
    fill_board_sector(board, Rot60Ccw);
    fill_board_sector(board, Rot120Ccw);

    board.insert(Hex { q: 7, r: 7 }, Cell::PlayerStart(0));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot60Cw), Cell::PlayerStart(1));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot120Cw), Cell::PlayerStart(2));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot180), Cell::PlayerStart(3));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot120Ccw), Cell::PlayerStart(4));
    board.insert(Hex { q: 7, r: 7 }.rotate(Rot60Ccw), Cell::PlayerStart(5));
}

fn fill_board_sector(board: &mut Board, rotation: hex::Rotation) {
    for q in 0..10 {
        for r in 0..BOARD_SECTOR[q] {
            board.insert(Hex { q: q as i32 + 1, r }.rotate(rotation), Cell::Empty);
        }
    }
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

fn pickup_shape(
    world_cursor: Res<CursorWorldCoords>,
    btn: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    hexagons: Query<(&Parent, &GlobalTransform), With<Selectable>>,
    shapes: Query<(&Transform, &Children), (With<HexShape>, Without<Selected>)>,
    selected_shape: Query<Entity, (With<HexShape>, With<Selected>)>,
    mut commands: Commands,
) {
    if !selected_shape.is_empty() || !btn.just_pressed(MouseButton::Left) {
        return;
    }

    for (parent, child_transform) in hexagons.iter() {
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

fn move_shape(
    mut scroll_evr: EventReader<MouseWheel>,
    world_cursor: Res<CursorWorldCoords>,
    mut game: ResMut<Game>,
    mut selected_shape: Query<&mut Transform, (With<HexShape>, With<Selected>)>,
    mut selected_hexagons: Query<(&GlobalTransform, &mut Sprite), (With<Selectable>, With<Selected>)>,
) {
    if let Ok(mut shape_transform) = selected_shape.get_single_mut() {
        for ev in scroll_evr.read() {
            let mut angle = 0.0_f32;
            if ev.y > 0.0 {
                angle = 60.0_f32.to_radians();
            } else if ev.y < 0.0 {
                angle = -60.0_f32.to_radians();
            }

            game.mouse_offset = Quat::from_rotation_z(angle).mul_vec3(game.mouse_offset.extend(0.0)).xy();
            shape_transform.rotate(Quat::from_rotation_z(angle));
        }

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

fn put_shape(
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

enum PutShapeAction {
    PutOnBoard,
    ReturnToOrigin,
    PutOutsideBoard
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
    let current_player_cell = Cell::Player(current_player);

    for hex in shape_hexes {
        // always can place on own starting square
        match board.get(hex) {
            Some(Cell::PlayerStart(index)) if *index == current_player => return true,
            _ => ()
        }

        // can't place when have direct neighbours
        if NEIGHBOURS.iter().any(|n|
            *board.get(&hex.add(n)).unwrap_or(&Cell::Empty) == current_player_cell
        ) {
            return false;
        }

        // can place when diagonals contains own squares
        if DIAGONAL_NEIGHBOURS.iter().any(|n|
            *board.get(&hex.add(n)).unwrap_or(&Cell::Empty) == current_player_cell
        ) {
            return true;
        }
    }

    false
}

fn board_system(
    mut board_hexes: Query<(&mut Sprite, &Hex), With<BoardHex>>,
    game: Res<Game>,
    selected_hexagons: Query<&GlobalTransform, (With<Selectable>, With<Selected>)>,
) {
    for (mut sprite, hex) in &mut board_hexes {
        if !selected_hexagons.is_empty() {
            let mut selected = false;
            for hex_position in selected_hexagons.iter() {
                let selected_hex = pixel_to_hex(hex_position.translation().xy());
                if selected_hex == *hex {
                    sprite.color = Color::GRAY;
                    selected = true;
                }
            }
            if selected {
                continue;
            }
        }

        if let Some(cell) = game.board.get(hex) {
            match cell {
                Cell::Empty => sprite.color = Color::WHITE,
                Cell::Player(i) => {
                    sprite.color = player_color(*i);
                },
                Cell::PlayerStart(i) => {
                    sprite.color = player_color_darken(*i);
                }
            };
        }
    }
}

fn hex_collision_with_point(point: Vec2, translation: Vec3) -> bool{
    translation.xy().distance_squared(point) <= HEX_RADIUS * HEX_RADIUS
}

fn world_cursor_system(
    mut world_cursor: ResMut<CursorWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        world_cursor.0 = world_position;
    }
}