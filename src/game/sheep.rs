//! Sheep behavior and spawning.

use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        level::{GOAL_RADIUS, GoalLocation},
        movement::HopMovementController,
        player::Player,
        state::GameState,
    },
    screens::Screen,
};

const SHEEP_INTERACT_RANGE: f32 = 5.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SheepAssets>();
    app.add_systems(
        Update,
        (sheep_goal_check, sheep_state_update, sheep_wander)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SheepState {
    Wander(Timer),
    Evading(Vec2),
    Spooked(Vec2),
    BeingCounted,
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Sheep {
    state: SheepState,
    step_distance: f32,
    min_wait: f32,
    max_wait: f32,
    spooked_speed_mult: f32,
}

impl Sheep {
    fn new() -> Self {
        let mut wander = Self {
            state: SheepState::Wander(Timer::from_seconds(1.0, TimerMode::Once)),
            step_distance: 2.0,
            min_wait: 0.6,
            max_wait: 2.0,
            spooked_speed_mult: 2.0,
        };
        wander.reset_timer();
        wander
    }

    fn reset_timer(&mut self) {
        if let SheepState::Wander(timer) = &mut self.state {
            let rng = &mut rand::rng();
            let wait = rng.random_range(self.min_wait..self.max_wait);
            timer.set_duration(Duration::from_secs_f32(wait));
            timer.reset();
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SheepAssets {
    #[dependency]
    pub scene: Handle<Scene>,
}

impl FromWorld for SheepAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            scene: assets.load("obj/sheep.glb#Scene0"),
        }
    }
}

pub fn spawn_sheep(sheep_assets: &SheepAssets, position: Vec3) -> impl Bundle {
    (
        Name::new("Sheep"),
        Sheep::new(),
        HopMovementController::default(),
        SceneRoot(sheep_assets.scene.clone()),
        Transform::from_translation(position),
        DespawnOnExit(Screen::Gameplay),
    )
}

fn sheep_wander(
    time: Res<Time>,
    mut sheep_query: Query<(&mut HopMovementController, &Transform, &mut Sheep)>,
) {
    for (mut controller, transform, mut sheep) in &mut sheep_query {
        if let SheepState::Wander(timer) = &mut sheep.state {
            timer.tick(time.delta());
            if timer.just_finished() {
                let rng = &mut rand::rng();
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let dir = Vec2::from_angle(angle);
                let target = transform.translation.xz() + dir * sheep.step_distance;
                controller.intent = target;
                sheep.reset_timer();
            }
        }
    }
}

fn sheep_state_update(
    time: Res<Time>,
    mut sheep_query: Query<(&mut HopMovementController, &Transform, &mut Sheep)>,
    player_query: Query<&Transform, With<Player>>,
    goal_query: Query<&Transform, (With<GoalLocation>, Without<Player>)>,
) {
    for (mut controller, transform, mut sheep) in &mut sheep_query {
        let pos = transform.translation.xz();
        match sheep.state {
            SheepState::Wander(_) => {
                for player_transform in player_query {
                    let player_pos = player_transform.translation.xz();
                    if pos.distance(player_pos) < SHEEP_INTERACT_RANGE {
                        sheep.state = SheepState::Evading(player_pos);
                    }
                }
            }
            SheepState::Evading(danger_pos) => {
                if pos.distance(danger_pos) >= SHEEP_INTERACT_RANGE {
                    sheep.state = SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                    sheep.reset_timer();
                } else {
                    let dir = (pos - danger_pos).normalize_or(Vec2::X);
                    controller.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                }
            }
            SheepState::Spooked(danger_pos) => {
                if pos.distance(danger_pos) >= SHEEP_INTERACT_RANGE * 2.0 {
                    sheep.state = SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                    sheep.reset_timer();
                } else {
                    let dir = (pos - danger_pos).normalize_or(Vec2::X);
                    controller.hop_speed_mult = sheep.spooked_speed_mult;
                    controller.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                }
            }
            SheepState::BeingCounted => {
                let goal_pos = goal_query.single().unwrap().translation.xz();
                let dir = (goal_pos - pos).normalize_or(Vec2::X);
                controller.hop_speed_mult = 0.5;
                controller.apply_movement(dir * time.delta_secs() * sheep.step_distance);
            }
        }
    }
}

fn sheep_goal_check(
    mut commands: Commands,
    sheep_query: Query<(Entity, &Transform, &mut Sheep)>,
    goal_query: Single<&Transform, With<GoalLocation>>,
    mut state: ResMut<GameState>,
) {
    let goal_pos = goal_query.translation.xz();
    for (entity, sheep_transform, mut sheep) in sheep_query {
        let pos = sheep_transform.translation.xz();
        if let SheepState::BeingCounted = sheep.state {
            if pos.distance_squared(goal_pos) < 1.5 {
                state.points += 1;
                info!("Points: {}", state.points);
                commands.entity(entity).despawn();
            }
        } else {
            if pos.distance_squared(goal_pos) < GOAL_RADIUS * GOAL_RADIUS {
                sheep.state = SheepState::BeingCounted;
            }
        }
    }
}
