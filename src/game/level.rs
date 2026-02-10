//! Spawn the main level.

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, game::player::player, screens::Screen};

pub const GOAL_RADIUS: f32 = 6.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
    app.insert_resource(LevelBounds {
        min: (-34.5, -49.5).into(),
        max: (34.5, 9.5).into(),
    });
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct LevelBounds {
    pub max: Vec2,
    pub min: Vec2,
}

impl LevelBounds {
    pub fn clamp_to_bounds(&self, pos: Vec2) -> Vec2 {
        Vec2::new(
            pos.x.clamp(self.min.x, self.max.x),
            pos.y.clamp(self.min.y, self.max.y),
        )
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    // #[dependency]
    // music: Handle<AudioSource>,
    #[dependency]
    arena: Handle<Scene>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            // music: assets.load("audio/music/Fluffing A Duck.ogg"),
            arena: assets.load("obj/arena.glb#Scene0"),
        }
    }
}

#[derive(Component)]
pub struct GoalLocation;

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    level_assets: Res<LevelAssets>,
) {
    let mut gizmo = GizmoAsset::new();

    gizmo
        .sphere(
            Isometry3d::IDENTITY,
            GOAL_RADIUS,
            bevy::color::palettes::css::CRIMSON,
        )
        .resolution(30_000 / 6);

    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            // (
            //     Name::new("Gameplay Music"),
            //     music(level_assets.music.clone())
            // ),
            SceneRoot(level_assets.arena.clone()),
            (
                Name::new("Sun"),
                DirectionalLight {
                    shadows_enabled: true,
                    color: Color::srgb(0.284, 0.358, 0.659),
                    ..Default::default()
                },
                Transform::from_xyz(0.5, 0.3, 1.0).looking_at(Vec3::ZERO, Vec3::Y)
            ),
            (
                Name::new("Goal"),
                GoalLocation,
                Transform::from_xyz(0.0, 0.0, 10.0),
                Gizmo {
                    handle: gizmo_assets.add(gizmo),
                    line_config: GizmoLineConfig {
                        width: 0.5,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
        ],
    ));
}
