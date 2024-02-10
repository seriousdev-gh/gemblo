use bevy::render::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

#[derive(Component)]
struct Collider;


#[derive(Component)]
struct HexShape;

#[derive(Bundle)]
struct HexBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl HexBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
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
    dragging: bool,
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
        .add_systems(Update, (my_cursor_system, game_update))
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
        parent.spawn(HexBundle::new(HEX_UP + HEX_UP_LEFT, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_UP, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_DOWN, hex_texture_handle));
        parent.spawn(HexBundle::new(HEX_DOWN + HEX_DOWN_RIGHT, hex_texture_handle));
    });
}

fn game_update(
    world_cursor: Res<CursorWorldCoords>, 
    btn: Res<Input<MouseButton>>, 
    game: ResMut<Game>,
    free_hexes: Query<&Transform, With<Collider>>,
    mut gizmos: Gizmos,
) {
    if !game.dragging {
        free_hexes.for_each(|f| { 
            if hex_collision_with_point(world_cursor.0, f) {
                gizmos.circle_2d(f.translation.xy(), HEX_RADIUS, Color::RED);
            }
         } );
    }
}

fn hex_collision_with_point(point: Vec2, transform: &Transform) -> bool{
    transform.translation.xy().distance_squared(point) <= HEX_RADIUS * HEX_RADIUS
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