//! Spawn the main level.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::{MusicLayer, music},
    game::camera::MainCamera,
    screens::Screen,
};

pub const GOAL_RADIUS: f32 = 6.0;
const GOAL_TEXT_LIFETIME_SECS: f32 = 2.5;
const GOAL_TEXT_RISE_SPEED: f32 = 0.8;
const GOAL_TEXT_FONT_SIZE: f32 = 32.0;
const GOAL_TEXT_HEIGHT_OFFSET: f32 = 1.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
    app.add_message::<GoalTextMessage>();
    app.add_systems(
        Update,
        (spawn_goal_text, tick_goal_text)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.insert_resource(LevelBounds {
        min: (-27.6, -39.6).into(),
        max: (27.6, 7.6).into(),
    });
    app.add_observer(handle_random_teleport);
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
    #[dependency]
    bgm_layer_1: Handle<AudioSource>,
    #[dependency]
    bgm_layer_2: Handle<AudioSource>,
    #[dependency]
    bgm_layer_3: Handle<AudioSource>,
    #[dependency]
    arena: Handle<Scene>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            bgm_layer_1: assets.load("audio/music/bgm_layer_1.ogg"),
            bgm_layer_2: assets.load("audio/music/bgm_layer_2.ogg"),
            bgm_layer_3: assets.load("audio/music/bgm_layer_3.ogg"),
            arena: assets.load("obj/arena.glb#Scene0"),
        }
    }
}

#[derive(EntityEvent)]
pub struct RandomTeleport {
    pub entity: Entity,
}

fn handle_random_teleport(
    event: On<RandomTeleport>,
    mut query: Query<&mut Transform>,
    bounds: Res<LevelBounds>,
) {
    if let Ok(mut transform) = query.get_mut(event.entity) {
        let rng = &mut rand::rng();
        let x = rng.random_range(bounds.min.x..=bounds.max.x);
        let z = rng.random_range(bounds.min.y..=bounds.max.y);
        let pos = Vec3::new(x, 0.0, z);
        transform.translation = pos;
    }
}

#[derive(Component)]
pub struct GoalLocation;

#[derive(Message, Debug, Clone)]
pub struct GoalTextMessage {
    pub text: String,
    pub color: Option<Color>,
}

#[derive(Component, Debug)]
struct GoalFloatingText {
    world_pos: Vec3,
    lifetime: Timer,
}

fn spawn_goal_text(
    mut commands: Commands,
    mut events: MessageReader<GoalTextMessage>,
    goal_query: Query<&GlobalTransform, With<GoalLocation>>,
) {
    let Some(goal_transform) = goal_query.iter().next() else {
        return;
    };

    for event in events.read() {
        commands.spawn((
            Name::new("Goal Floating Text"),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Text::new(event.text.clone()),
            TextFont::from_font_size(GOAL_TEXT_FONT_SIZE),
            TextColor(event.color.unwrap_or(Color::WHITE)),
            Pickable::IGNORE,
            GoalFloatingText {
                world_pos: goal_transform.translation() + Vec3::Y * GOAL_TEXT_HEIGHT_OFFSET,
                lifetime: Timer::from_seconds(GOAL_TEXT_LIFETIME_SECS, TimerMode::Once),
            },
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

fn tick_goal_text(
    mut commands: Commands,
    time: Res<Time>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query: Query<(Entity, &mut Node, &mut GoalFloatingText)>,
) {
    let (camera, camera_transform) = *camera;

    for (entity, mut node, mut floating_text) in &mut query {
        floating_text.lifetime.tick(time.delta());
        floating_text.world_pos.y += GOAL_TEXT_RISE_SPEED * time.delta_secs();

        match camera.world_to_viewport(camera_transform, floating_text.world_pos) {
            Ok(viewport_pos) => {
                node.left = px(viewport_pos.x);
                node.top = px(viewport_pos.y);
                node.display = Display::DEFAULT;
            }
            Err(_) => {
                node.display = Display::None;
            }
        }

        if floating_text.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    // mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    level_assets: Res<LevelAssets>,
) {
    // let mut gizmo = GizmoAsset::new();

    // gizmo
    //     .sphere(
    //         Isometry3d::IDENTITY,
    //         GOAL_RADIUS,
    //         bevy::color::palettes::css::CRIMSON,
    //     )
    //     .resolution(30_000 / 6);

    commands.spawn((
        Name::new("Level"),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        Transform::default(),
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
                    color: Color::srgb(0.9, 1.0, 0.9),
                    ..Default::default()
                },
                Transform::from_xyz(0.5, 0.5, 2.0).looking_at(Vec3::ZERO, Vec3::Y)
            ),
            (
                Name::new("Goal"),
                GoalLocation,
                Transform::from_xyz(0.0, 0.0, 8.0),
                // Gizmo {
                //     handle: gizmo_assets.add(gizmo),
                //     line_config: GizmoLineConfig {
                //         width: 0.5,
                //         ..Default::default()
                //     },
                //     ..Default::default()
                // }
            )
        ],
    ));
}

pub fn start_music(mut commands: Commands, assets: Res<LevelAssets>) {
    commands.spawn((
        music(assets.bgm_layer_1.clone()),
        MusicLayer::Base,
        DespawnOnExit(Screen::Gameplay),
    ));
    commands.spawn((
        music(assets.bgm_layer_2.clone()),
        MusicLayer::Extra,
        DespawnOnExit(Screen::Gameplay),
    ));
    commands.spawn((
        music(assets.bgm_layer_3.clone()),
        MusicLayer::Perc,
        DespawnOnExit(Screen::Gameplay),
    ));
}
