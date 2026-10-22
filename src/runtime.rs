//! Bevy host application.
//!
//! This is only the host-window scaffold. The rectangle is a placeholder,
//! not a rendered GBA framebuffer; the cartridge model and generated game
//! module are not implemented yet.

use bevy::prelude::*;

/// Construct the application without entering its event loop.
///
/// Keeping construction separate makes app configuration testable without
/// opening a real window.
pub fn build_app() -> App {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GBARecomp Rust".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.065)))
        .add_systems(Startup, setup_host_scene)
}

fn setup_host_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.02, 0.025, 0.04), Vec2::new(720.0, 480.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// Start the Bevy host event loop.
pub fn run() {
    build_app().run();
}
