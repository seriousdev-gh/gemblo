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
struct Board;

#[derive(Component)]
struct BoardHex;

#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct HexShape;

const DEFAULT_Z: f32 = 1.0;
const SELECTED_Z: f32 = 1.0001;

fn build_selectable_hex(location: Vec2, texture: &Handle<Image>, color: Color) -> (SpriteBundle, Selectable) {
    (
        build_hex(location, texture, color),
        Selectable
    )
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

const HEX_SCALE: f32 = 0.25;
const HEX_REAL_WIDTH_IN_PIXELS: f32 = 128.0;
const HEX_WIDTH: f32 = HEX_REAL_WIDTH_IN_PIXELS * HEX_SCALE;
const HEX_RADIUS: f32 = HEX_WIDTH / 2.0;

const HEX_UP:Vec2       = Vec2 { x: 0.0, y: HEX_WIDTH * SQRT_3 / 2.0 };
const HEX_DOWN:Vec2     = Vec2 { x: 0.0, y: -HEX_WIDTH * SQRT_3 / 2.0 };
const HEX_UP_LEFT:Vec2  = Vec2 { x: -HEX_WIDTH * 3.0 / 4.0 , y: HEX_WIDTH * SQRT_3 / 4.0 };
const HEX_UP_RIGHT:Vec2 = Vec2 { x: HEX_WIDTH * 3.0 / 4.0, y: HEX_WIDTH * SQRT_3 / 4.0 };
const HEX_DOWN_LEFT:Vec2  = Vec2 { x: -HEX_WIDTH * 3.0 / 4.0, y: -HEX_WIDTH * SQRT_3 / 4.0 };
const HEX_DOWN_RIGHT:Vec2 = Vec2 { x: HEX_WIDTH * 3.0 / 4.0, y: -HEX_WIDTH * SQRT_3 / 4.0 };

enum Cell {
    Empty,
    Player1,
    Player2,
    Player3,
    Player4,
    Player5,
    Player6
}

#[derive(Resource, Default)]
struct Game {
    original_transform: Transform,
    mouse_offset: Vec2,
    board: HashMap<Hex, Cell>
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
        .insert_resource(Game { ..default() })
        .insert_resource(CursorWorldCoords { ..default() })
        .add_systems(Startup, setup)
        .add_systems(Update, (world_cursor_system, pickup_shape, move_shape, put_shape, board_system))
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

    commands.spawn((
        HexShape,
        SpatialBundle { transform: Transform::from_xyz(HEX_WIDTH * 15., 0., 0.), ..default() }
    )).with_children(|parent| {
        parent.spawn(build_selectable_hex(Vec2::ZERO, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(HEX_UP + HEX_UP_RIGHT, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(HEX_UP, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(HEX_DOWN, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(HEX_DOWN + HEX_DOWN_LEFT, hex_texture_handle, Color::GREEN));
    });


    commands.spawn((
        HexShape,
        SpatialBundle { transform: Transform::from_xyz(HEX_WIDTH * 15., 256., 0.), ..default() }
    )).with_children(|parent| {
        parent.spawn(build_selectable_hex(HEX_UP, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(Vec2::ZERO, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(HEX_DOWN, hex_texture_handle, Color::GREEN));
    });


    commands.spawn((
        HexShape,
        SpatialBundle { transform: Transform::from_xyz(HEX_WIDTH * 15., -256., 0.), ..default() }
    )).with_children(|parent| {
        parent.spawn(build_selectable_hex(HEX_UP_LEFT, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(Vec2::ZERO, hex_texture_handle, Color::GREEN));
        parent.spawn(build_selectable_hex(HEX_DOWN_LEFT, hex_texture_handle, Color::GREEN));
    });

    fill_board(&mut game.board);

    commands.spawn((Board, SpatialBundle::default())).with_children(|parent| {
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
}

fn fill_board(board: &mut HashMap<Hex, Cell>) {
    board.insert(Hex { q: 0, r: 0 }, Cell::Empty);
    fill_board_sector(board, hex::Rotation::Rot0);
    fill_board_sector(board, hex::Rotation::Rot60Cw);
    fill_board_sector(board, hex::Rotation::Rot120Cw);
    fill_board_sector(board, hex::Rotation::Rot180);
    fill_board_sector(board, hex::Rotation::Rot60Ccw);
    fill_board_sector(board, hex::Rotation::Rot120Ccw);
}

fn fill_board_sector(board: &mut HashMap<Hex, Cell>, rotation: hex::Rotation) {
    for q in 0..10 {
        for r in 0..BOARD_SECTOR[q] {
            board.insert(Hex { q: q as i32 + 1, r }.rotate(rotation), Cell::Empty);
        }
    }
}

fn hex_to_pixel(hex: &Hex) -> Vec2 {
    Vec2 {
        x: HEX_RADIUS * (3./2. * hex.q as f32),
        y: HEX_RADIUS * (SQRT_3/2. * hex.q as f32  + SQRT_3 * hex.r as f32)
    }
}

fn pixel_to_hex(pixel: Vec2) -> Hex {
    let q = ( 2./3.0 * pixel.x) / HEX_RADIUS;
    let r = (-1./3.0 * pixel.x + SQRT_3/3.0 * pixel.y) / HEX_RADIUS;
    Hex::from_fraction(q, r)
}

fn pickup_shape(
    world_cursor: Res<CursorWorldCoords>,
    btn: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    hexagons: Query<(&Parent, &GlobalTransform), With<Selectable>>,
    shapes: Query<(&Transform, &Children), (With<HexShape>, Without<Selected>)>,
    selected_shape: Query<&Transform, (With<HexShape>, With<Selected>)>,
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
    mut selected_shape: Query<&mut Transform, (With<HexShape>, With<Selected>)>
) {
    use bevy::input::mouse::MouseScrollUnit;
    if let Ok(mut shape_transform) = selected_shape.get_single_mut() {
        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    let mut angle = 0.0_f32;
                    if ev.y > 0.0 {
                        angle = 60.0_f32.to_radians();
                    } else if ev.y < 0.0 {
                        angle = -60.0_f32.to_radians();
                    }

                    game.mouse_offset = Quat::from_rotation_z(angle).mul_vec3(game.mouse_offset.extend(0.0)).xy();
                    shape_transform.rotate(Quat::from_rotation_z(angle));
                }
                MouseScrollUnit::Pixel => {
                    todo!();
                }
            }
        }

        shape_transform.translation = Vec3 {
            x: game.mouse_offset.x + world_cursor.0.x,
            y: game.mouse_offset.y + world_cursor.0.y,
            z: SELECTED_Z
        };
    }
}

fn put_shape(
    btn: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    selected_hexagons: Query<(Entity, &GlobalTransform), (With<Selectable>, With<Selected>)>,
    mut selected_shape: Query<(Entity, &mut Transform), (With<HexShape>, With<Selected>)>,
    mut commands: Commands,
) {
    if btn.pressed(MouseButton::Left) {
        return;
    }

    if let Ok(mut shape) = selected_shape.get_single_mut() {
        let rounded_shape_hexes: Vec<Hex> = selected_hexagons.iter().map(|f|
            pixel_to_hex(f.1.translation().xy())
        ).collect();
        let shape_status = shape_contained_on_board(&game.board, &rounded_shape_hexes);

        match shape_status {
            ShapeOnBoard::Full => {
                println!("Full");
                for hex in rounded_shape_hexes {
                    game.board.insert(hex, Cell::Player1);
                }
                for hex1 in selected_hexagons.iter() {
                    commands.entity(hex1.0).despawn();
                }
                commands.entity(shape.0).despawn();
            },
            ShapeOnBoard::Partial => {
                println!("Partial");
                shape.1.translation = game.original_transform.translation;
                shape.1.rotation = game.original_transform.rotation;
            },
            ShapeOnBoard::None => {
                println!("None");
            }
        }

        for hex1 in selected_hexagons.iter() {
            shape.1.translation.z = DEFAULT_Z;
            commands.entity(hex1.0).remove::<Selected>();
        }
        commands.entity(shape.0).remove::<Selected>();
    }
}

enum ShapeOnBoard {
    Full,
    Partial,
    None
}

fn shape_contained_on_board(board: &HashMap<Hex, Cell>, shape_hexes: &[Hex]) -> ShapeOnBoard {
    let hexes_on_board = shape_hexes.iter().filter(|hex| board.contains_key(hex)).count();
    if hexes_on_board == shape_hexes.len() {
        ShapeOnBoard::Full
    } else if hexes_on_board > 0 && hexes_on_board < shape_hexes.len() {
        ShapeOnBoard::Partial
    } else {
        ShapeOnBoard::None
    }
}

fn board_system(
    mut board_hexes: Query<(&mut Sprite, &Hex), With<BoardHex>>,
    game: Res<Game>,
    selected_hexagons: Query<&GlobalTransform, (With<Selectable>, With<Selected>)>,
) {
    for (mut sprite, hex) in &mut board_hexes {
        if !selected_hexagons.is_empty()  {
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
                Cell::Player1 => sprite.color = Color::GREEN,
                _ => todo!(),
            };
        }
    }
}

fn hex_collision_with_point(point: Vec2, translation: Vec3) -> bool{
    translation.xy().distance_squared(point) <= HEX_RADIUS * HEX_RADIUS
}

fn hex_collision_with_hex(point1: Vec3, point2: Vec3) -> bool{
    point1.distance_squared(point2) <= HEX_RADIUS * HEX_RADIUS * 2.0
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