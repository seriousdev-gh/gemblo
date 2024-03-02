#![feature(more_float_constants)]

mod hex;
mod ui;
mod game;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::ui::ui_plugin;
use crate::game::GamePlugin;

#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

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
        .insert_resource(CursorWorldCoords { ..default() })
        .add_plugins((ui_plugin, GamePlugin { player_count: 6 }))
        .add_systems(Update, world_cursor_system)
        .run();
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