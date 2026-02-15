use bevy::prelude::*;
use rand::seq::IteratorRandom;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        modifiers::Modifier,
        movement::{HopMovementController, MovementController, SphereMovementController},
        sheep::Sheep,
        state::{GamePhase, GameState},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<UfoAssets>();
    app.add_systems(OnEnter(GamePhase::Herding), spawn_ufo);
    app.add_systems(
        Update,
        tick_abduction_timers
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems)
            .run_if(in_state(GamePhase::Herding)),
    );
    app.add_systems(
        Update,
        (pick_targets, update_ufo)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(GamePhase::Herding)),
    );
}

pub const UFO_HEIGHT: f32 = 15.0;
const UFO_ABDUCTION_SECONDS: f32 = 8.0;
const UFO_POST_ABDUCTION_PAUSE_SECONDS: f32 = 3.0;
const UFO_SPEED: f32 = 7.0;
const UFO_TARGET_REACHED_DISTANCE: f32 = 0.5;

#[derive(Debug, Component)]
struct Ufo {
    abduction_timer: Timer,
    post_abduction_pause_timer: Timer,
    target: Option<Entity>,
}
impl Ufo {
    pub fn new() -> Self {
        let mut post_abduction_pause_timer =
            Timer::from_seconds(UFO_POST_ABDUCTION_PAUSE_SECONDS, TimerMode::Once);
        post_abduction_pause_timer.set_elapsed(post_abduction_pause_timer.duration());
        Self {
            abduction_timer: Timer::from_seconds(UFO_ABDUCTION_SECONDS, TimerMode::Once),
            post_abduction_pause_timer,
            target: None,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct UfoAssets {
    #[dependency]
    ufo: Handle<Scene>,
}

impl FromWorld for UfoAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ufo: assets.load("obj/ufo.glb#Scene0"),
        }
    }
}

fn spawn_ufo(mut commands: Commands, assets: Res<UfoAssets>, game_state: Res<GameState>) {
    if game_state.is_modifier_active(Modifier::Ufo) {
        commands.spawn((
            Name::new("UFO"),
            Transform::from_xyz(0.0, UFO_HEIGHT, -20.0),
            SceneRoot(assets.ufo.clone()),
            Ufo::new(),
            DespawnOnExit(GamePhase::Herding),
        ));
        if game_state.is_modifier_active(Modifier::FeverDream) {
            commands.spawn((
                Name::new("UFO"),
                Transform::from_xyz(0.0, UFO_HEIGHT, -20.0),
                SceneRoot(assets.ufo.clone()),
                Ufo::new(),
                DespawnOnExit(GamePhase::Herding),
            ));
        }
    }
}

fn tick_abduction_timers(time: Res<Time>, mut ufo_query: Query<&mut Ufo>) {
    for mut ufo in &mut ufo_query {
        ufo.abduction_timer.tick(time.delta());
        ufo.post_abduction_pause_timer.tick(time.delta());
    }
}

fn pick_targets(mut ufo_query: Query<&mut Ufo>, sheep_query: Query<Entity, With<Sheep>>) {
    for mut ufo in &mut ufo_query {
        if !ufo.post_abduction_pause_timer.is_finished() {
            ufo.target = None;
            continue;
        }
        if ufo.target.is_some() {
            continue;
        }

        let rng = &mut rand::rng();
        ufo.target = sheep_query.iter().choose(rng);
    }
}

fn update_ufo(
    time: Res<Time>,
    mut commands: Commands,
    mut ufo_query: Query<(&mut Transform, &mut Ufo), Without<Sheep>>,
    mut sheep_query: Query<(&Transform, &mut Sheep), Without<Ufo>>,
) {
    for (mut ufo_transform, mut ufo) in &mut ufo_query {
        ufo_transform.translation.y = UFO_HEIGHT;
        if !ufo.post_abduction_pause_timer.is_finished() {
            ufo.target = None;
            continue;
        }

        let Some(target) = ufo.target else {
            continue;
        };

        let Ok((target_transform, mut sheep)) = sheep_query.get_mut(target) else {
            ufo.target = None;
            continue;
        };

        let target_pos = target_transform.translation.xz();
        let ufo_pos = ufo_transform.translation.xz();
        let to_target = target_pos - ufo_pos;
        let distance = to_target.length();

        if distance > f32::EPSILON {
            let step = (UFO_SPEED * time.delta_secs()).min(distance);
            let dir = to_target / distance;
            ufo_transform.translation.x += dir.x * step;
            ufo_transform.translation.z += dir.y * step;
        }

        if distance <= UFO_TARGET_REACHED_DISTANCE {
            if ufo.abduction_timer.is_finished() {
                if sheep.start_abduction() {
                    commands.entity(target).remove::<(
                        MovementController,
                        HopMovementController,
                        SphereMovementController,
                    )>();
                    ufo.abduction_timer.reset();
                    ufo.post_abduction_pause_timer.reset();
                }
                ufo.target = None;
            } else {
                ufo.target = None;
            }
        }
    }
}
