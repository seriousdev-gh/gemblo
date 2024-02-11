use bevy::render::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct HexShape;

#[derive(Bundle)]
struct HexBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

const DEFAULT_Z: f32 = 1.0;
const SELECTED_Z: f32 = 1.0001;

impl HexBundle {
    fn new(location: Vec2, texture: &Handle<Image>) -> HexBundle {
        HexBundle {
                sprite_bundle: SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(0.0, 1.0, 0.0, 1.0),
                        ..default()
                    },
                    texture: texture.clone(),
                    transform: Transform::from_xyz(location.x, location.y, 0.).with_scale(Vec3 {x: HEX_SCALE, y: HEX_SCALE, z: 1.0 }),
                    ..default()
                },
                collider: Collider
            }
    }
}

const HEX_SCALE: f32 = 0.5;
const HEX_REAL_WIDTH_IN_PIXELS: f32 = 128.0;
const HEX_WIDTH: f32 = HEX_REAL_WIDTH_IN_PIXELS * HEX_SCALE;
const HEX_RADIUS: f32 = HEX_WIDTH / 2.0;

const SQRT3:f32 = 1.732_050_8;
const HEX_UP:Vec2       = Vec2 { x: 0.0, y: HEX_WIDTH * SQRT3 / 2.0 };
const HEX_DOWN:Vec2     = Vec2 { x: 0.0, y: -HEX_WIDTH * SQRT3 / 2.0 };
const HEX_UP_LEFT:Vec2  = Vec2 { x: -HEX_WIDTH * 3.0 / 4.0 , y: HEX_WIDTH * SQRT3 / 4.0 };
const HEX_UP_RIGHT:Vec2 = Vec2 { x: HEX_WIDTH * 3.0 / 4.0, y: HEX_WIDTH * SQRT3 / 4.0 };
const HEX_DOWN_LEFT:Vec2  = Vec2 { x: -HEX_WIDTH * 3.0 / 4.0, y: -HEX_WIDTH * SQRT3 / 4.0 };
const HEX_DOWN_RIGHT:Vec2 = Vec2 { x: HEX_WIDTH * 3.0 / 4.0, y: -HEX_WIDTH * SQRT3 / 4.0 };

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
    mouse_offset: Vec2
}

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
        .add_systems(Update, (my_cursor_system, pickup_shape, move_shape, put_shape))
        .run();
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::AutoMin { min_width: 1920.0, min_height: 1080.0 };
    commands.spawn(camera_bundle);

    let hex_texture_handle = &asset_server.load("hex.png");

    commands.spawn((
        HexShape,
        SpatialBundle { transform: Transform::from_xyz(0., 0., 0.), ..default() }
    )).with_children(|parent| {
        parent.spawn(HexBundle::new(Vec2::ZERO, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_UP + HEX_UP_RIGHT, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_UP, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_DOWN, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_DOWN + HEX_DOWN_LEFT, hex_texture_handle));
    });

    
    commands.spawn((
        HexShape,
        SpatialBundle { transform: Transform::from_xyz(256., 0., 0.), ..default() }
    )).with_children(|parent| {
        parent.spawn(HexBundle::new(HEX_UP, hex_texture_handle));
        parent.spawn(HexBundle::new(Vec2::ZERO, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_DOWN, hex_texture_handle));
    });

    
    commands.spawn((
        HexShape,
        SpatialBundle { transform: Transform::from_xyz(-256., 0., 0.), ..default() }
    )).with_children(|parent| {
        parent.spawn(HexBundle::new(HEX_UP_LEFT, hex_texture_handle));
        parent.spawn(HexBundle::new(Vec2::ZERO, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_DOWN_LEFT, hex_texture_handle));
    });
}

fn pickup_shape(
    world_cursor: Res<CursorWorldCoords>, 
    btn: Res<Input<MouseButton>>, 
    mut game: ResMut<Game>,
    hexagons: Query<(&Parent, &GlobalTransform), With<Collider>>,
    shapes: Query<(&Transform, &Children), (With<HexShape>, Without<Selected>)>,
    selected_shape: Query<&Transform, (With<HexShape>, With<Selected>)>,
    mut commands: Commands,
) {
    if selected_shape.is_empty() && btn.just_pressed(MouseButton::Left) {
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
}

fn move_shape(
    world_cursor: Res<CursorWorldCoords>,
    game: ResMut<Game>,
    mut selected_shape: Query<&mut Transform, (With<HexShape>, With<Selected>)>
) {
    if let Ok(mut shape_transform) = selected_shape.get_single_mut() {
        shape_transform.translation = Vec3 { 
            x: game.mouse_offset.x + world_cursor.0.x, 
            y: game.mouse_offset.y + world_cursor.0.y, 
            z: SELECTED_Z 
        };
    }
}


fn put_shape(
    btn: Res<Input<MouseButton>>, 
    game: ResMut<Game>,
    not_selected_hexagons: Query<&GlobalTransform, (With<Collider>, Without<Selected>)>,
    selected_hexagons: Query<(Entity, &GlobalTransform), (With<Collider>, With<Selected>)>,
    mut selected_shape: Query<(Entity, &mut Transform), (With<HexShape>, With<Selected>)>,
    mut commands: Commands,
) {
    if !btn.pressed(MouseButton::Left) {
        if let Ok(mut shape) = selected_shape.get_single_mut() {
            for hex1 in selected_hexagons.iter() {
                for hex2 in not_selected_hexagons.iter() {
                    if hex_collision_with_hex(hex1.1.translation(), hex2.translation()) {
                        shape.1.translation = game.original_transform.translation;
                        shape.1.rotation = game.original_transform.rotation;
                    }   
                }

                shape.1.translation.z = DEFAULT_Z;

                commands.entity(hex1.0).remove::<Selected>();                
            }
            commands.entity(shape.0).remove::<Selected>();  
        }
    }
}

fn hex_collision_with_point(point: Vec2, translation: Vec3) -> bool{
    translation.xy().distance_squared(point) <= HEX_RADIUS * HEX_RADIUS
}

fn hex_collision_with_hex(point1: Vec3, point2: Vec3) -> bool{
    point1.distance_squared(point2) <= HEX_RADIUS * HEX_RADIUS * 2.0
}

fn my_cursor_system(
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