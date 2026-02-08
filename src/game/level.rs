//! Spawn the main level.

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, game::player::player, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
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

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(meshes, materials),
            // (
            //     Name::new("Gameplay Music"),
            //     music(level_assets.music.clone())
            // ),
            SceneRoot(level_assets.arena.clone()),
            (
                Name::new("Sun"),
                DirectionalLight {
                    shadows_enabled: true,
                    ..Default::default()
                },
                Transform::from_xyz(0.5, 0.3, 1.0).looking_at(Vec3::ZERO, Vec3::Y)
            )
        ],
    ));
}
