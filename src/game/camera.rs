use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, game::player::Player};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CameraTarget>();
    app.init_resource::<CameraFollow>();
    app.add_systems(
        Update,
        (set_camera_target_to_player, move_camera_to_target)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Default)]
pub struct CameraTarget(pub Option<Entity>);

#[derive(Resource)]
struct CameraFollow {
    offset: Option<Vec3>,
    smoothing: f32,
}

impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            offset: None,
            smoothing: 8.0,
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

fn set_camera_target_to_player(
    mut target: ResMut<CameraTarget>,
    player_query: Query<Entity, With<Player>>,
) {
    if target.0.is_some() {
        return;
    }

    if let Ok(entity) = player_query.single() {
        target.0 = Some(entity);
    }
}

fn move_camera_to_target(
    time: Res<Time>,
    target: Res<CameraTarget>,
    mut follow: ResMut<CameraFollow>,
    target_query: Query<&Transform, Without<MainCamera>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Some(target_entity) = target.0 else {
        return;
    };

    let Ok(target_transform) = target_query.get(target_entity) else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let offset = follow
        .offset
        .get_or_insert_with(|| camera_transform.translation - target_transform.translation);
    let desired = target_transform.translation + *offset;

    let t = 1.0 - (-follow.smoothing * time.delta_secs()).exp();
    camera_transform.translation = camera_transform.translation.lerp(desired, t);
}
