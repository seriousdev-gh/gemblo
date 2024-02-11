use bevy::render::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct HexShape;

#[derive(Bundle)]
struct HexBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

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
    dragging: bool,
    move_to_original: bool,
    selected_id: Option<Entity>,
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
    free_hexes: Query<(Entity, &Parent, &GlobalTransform), With<Collider>>,
    shapes: Query<&Transform, With<HexShape>>,
    gizmos: Gizmos,
    commands: Commands,
) {
    // for (ent, parent, child_transform) in free_hexes.iter() {
    //     gizmos.circle_2d(child_transform.translation().xy(), HEX_RADIUS, Color::RED);
    // }
    if !game.dragging && btn.just_pressed(MouseButton::Left) {
        for (ent, parent, child_transform) in free_hexes.iter() {
            if hex_collision_with_point(world_cursor.0, child_transform.translation()) {
                println!("Clicked entity: {:?}, parent: {:?}", ent, parent.get());
                game.dragging = true;
                game.selected_id = Some(parent.get());
                // commands.entity(parent.get())
                //     .remove::<Enemy>();
                
                let parent_global_transform = shapes.get(parent.get());
                if let Ok(transform) = parent_global_transform {
                    game.original_transform = *transform;
                    game.mouse_offset = transform.translation.xy() - world_cursor.0;
                }
                return;
            }
         }
    }
}

fn move_shape(
    world_cursor: Res<CursorWorldCoords>,
    mut game: ResMut<Game>,
    mut shapes: Query<&mut Transform, With<HexShape>>
) {
    if game.dragging {
        if let Some(selected_id) = game.selected_id {
            let parent_global_transform = shapes.get_mut(selected_id);
            if let Ok(mut shape_transform) = parent_global_transform {
                shape_transform.translation = Vec3 { x: game.mouse_offset.x + world_cursor.0.x, y: game.mouse_offset.y + world_cursor.0.y, z: 0. };
            }
        }
    }

    if game.move_to_original {
        game.move_to_original = false;
        if let Some(selected_id) = game.selected_id {
            let parent_global_transform = shapes.get_mut(selected_id);
            if let Ok(mut shape_transform) = parent_global_transform {
                shape_transform.translation = game.original_transform.translation;
                shape_transform.rotation = game.original_transform.rotation;
            }
        }
    }
}


fn put_shape(
    world_cursor: Res<CursorWorldCoords>, 
    btn: Res<Input<MouseButton>>, 
    mut game: ResMut<Game>,
    free_hexes: Query<(&Parent, &GlobalTransform), With<Collider>>,
    shapes: Query<(&Transform, &Children), With<HexShape>>,
) {
    if game.dragging && !btn.pressed(MouseButton::Left) {

        println!("Trying to put piece");
        if let Some(selected_id) = game.selected_id {
            println!("Selected: {:?}", selected_id);
            if let Ok(selected_shape) = shapes.get(selected_id) {
                for child in selected_shape.1 {
                    if let Ok(child_hex) = free_hexes.get(*child) {
                        // selected_hex_transform.
                        for (hex_parent, hex_transform) in free_hexes.iter() {
                            
                            println!("Check if same piece: left: {:?}, right: {:?}", hex_parent.get(), selected_id);
                            if hex_parent.get() == selected_id {
                                continue;
                            }

                            println!("Check collision with: {:?}", hex_transform.translation());
                            if hex_collision_with_hex(child_hex.1.translation(), hex_transform.translation()) {
                                println!("Move to original");
                                game.move_to_original = true;
                            }   
                        }
                    }
                }
            }
        }

        game.dragging = false;
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