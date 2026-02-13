//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    game::{level::LevelBounds, movement::MovementController, state::GamePhase},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DebugGizmoOverlay>();
    app.add_plugins(EguiPlugin::default()).add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::F1)),
    );
    app.add_plugins(FreeCameraPlugin);
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
    app.add_systems(
        Update,
        spawn_debug_camera.run_if(input_just_pressed(KeyCode::F2)),
    );
    app.add_systems(Update, draw_level_bounds);
    app.add_systems(
        Update,
        skip_to_interlude.run_if(input_just_pressed(KeyCode::F3)),
    );
    app.add_systems(
        Update,
        toggle_intent_overlay.run_if(input_just_pressed(KeyCode::F4)),
    );
    app.add_systems(Update, draw_movement_intents);
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

#[derive(Resource, Default)]
struct DebugGizmoOverlay {
    enabled: bool,
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn toggle_intent_overlay(mut debug_gizmo_overlay: ResMut<DebugGizmoOverlay>) {
    debug_gizmo_overlay.enabled = !debug_gizmo_overlay.enabled;
}

fn spawn_debug_camera(mut commands: Commands, cameras: Query<Entity, With<Camera>>) {
    for entity in &cameras {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, 0.0).looking_to(Vec3::X, Vec3::Y),
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
    ));
}

fn draw_level_bounds(
    mut gizmos: Gizmos,
    bounds: Res<LevelBounds>,
    debug_gizmo_overlay: Res<DebugGizmoOverlay>,
) {
    if !debug_gizmo_overlay.enabled {
        return;
    }

    const HEIGHT_OFFSET: f32 = 0.5;
    let min = Vec3::new(bounds.min.x, HEIGHT_OFFSET, bounds.min.y);
    let max = Vec3::new(bounds.max.x, HEIGHT_OFFSET, bounds.max.y);
    let a = Vec3::new(min.x, HEIGHT_OFFSET, min.z);
    let b = Vec3::new(max.x, HEIGHT_OFFSET, min.z);
    let c = Vec3::new(max.x, HEIGHT_OFFSET, max.z);
    let d = Vec3::new(min.x, HEIGHT_OFFSET, max.z);
    let color = Color::srgb(0.9, 0.7, 0.2);

    gizmos.line(a, b, color);
    gizmos.line(b, c, color);
    gizmos.line(c, d, color);
    gizmos.line(d, a, color);
}

fn draw_movement_intents(
    mut gizmos: Gizmos,
    debug_gizmo_overlay: Res<DebugGizmoOverlay>,
    controllers: Query<(&Transform, &MovementController)>,
) {
    if !debug_gizmo_overlay.enabled {
        return;
    }

    const HEIGHT_OFFSET: f32 = 0.2;
    const MARKER_HALF_SIZE: f32 = 0.12;
    let line_color = Color::srgb(0.2, 0.9, 1.0);
    let marker_color = Color::srgb(1.0, 0.2, 0.2);

    for (transform, controller) in &controllers {
        let current = Vec3::new(
            transform.translation.x,
            transform.translation.y + HEIGHT_OFFSET,
            transform.translation.z,
        );
        let intent = Vec3::new(
            controller.intent.x,
            transform.translation.y + HEIGHT_OFFSET,
            controller.intent.y,
        );
        gizmos.line(current, intent, line_color);

        gizmos.line(
            intent + Vec3::new(-MARKER_HALF_SIZE, 0.0, 0.0),
            intent + Vec3::new(MARKER_HALF_SIZE, 0.0, 0.0),
            marker_color,
        );
        gizmos.line(
            intent + Vec3::new(0.0, 0.0, -MARKER_HALF_SIZE),
            intent + Vec3::new(0.0, 0.0, MARKER_HALF_SIZE),
            marker_color,
        );
    }
}

fn skip_to_interlude(mut next_state: ResMut<NextState<GamePhase>>) {
    next_state.set(GamePhase::ModifierChoice);
}
