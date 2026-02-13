//! Sheep behavior and spawning.

use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        level::{GOAL_RADIUS, GoalLocation, LevelBounds},
        modifiers::Modifier,
        movement::{HopMovementController, MovementController},
        player::Player,
        state::GameState,
        ufo::UFO_HEIGHT,
    },
    screens::Screen,
};

const SHEEP_INTERACT_RANGE: f32 = 5.0;
const ABDUCTION_ASCENT_SPEED: f32 = 6.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SheepAssets>();
    app.add_systems(
        Update,
        (
            sheep_goal_check,
            sheep_state_update,
            (sheep_wander, sheep_abduction_update),
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SheepState {
    Wander(Timer),
    /// Player is nearby - move away from them
    Evading(Vec2),
    /// Player barked - run away
    Spooked(Vec2),
    /// Near the goal - move towards it
    BeingCounted,
    /// Targeted by UFO - rise into the sky.
    BeingAbducted,
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Sheep {
    state: SheepState,
    step_distance: f32,
    min_wait: f32,
    max_wait: f32,
    default_speed_mult: f32,
    spooked_speed_mult: f32,
}

impl Sheep {
    fn new() -> Self {
        let mut sheep = Self {
            state: SheepState::Wander(Timer::from_seconds(1.0, TimerMode::Once)),
            step_distance: 2.0,
            min_wait: 1.5,
            max_wait: 5.0,
            default_speed_mult: 1.0,
            spooked_speed_mult: 2.0,
        };
        sheep.reset_timer();
        sheep
    }

    fn default_speed_mult(mut self, mult: f32) -> Self {
        self.default_speed_mult = mult;
        self
    }

    fn spooked_speed_mult(mut self, mult: f32) -> Self {
        self.spooked_speed_mult = mult;
        self
    }

    fn step_distance(mut self, dist: f32) -> Self {
        self.step_distance = dist;
        self
    }

    fn reset_timer(&mut self) {
        if let SheepState::Wander(timer) = &mut self.state {
            let rng = &mut rand::rng();
            let wait = rng.random_range(self.min_wait..self.max_wait);
            timer.set_duration(Duration::from_secs_f32(wait));
            timer.reset();
        }
    }

    pub fn become_spooked(&mut self, danger_pos: Vec2) {
        match self.state {
            SheepState::Wander(_) | SheepState::Evading(_) => {
                self.state = SheepState::Spooked(danger_pos);
            }
            _ => {}
        }
    }

    pub fn is_being_abducted(&self) -> bool {
        matches!(self.state, SheepState::BeingAbducted)
    }

    pub fn start_abduction(&mut self) -> bool {
        if self.is_being_abducted() {
            return false;
        }
        self.state = SheepState::BeingAbducted;
        true
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

pub fn sheep(sheep_assets: &SheepAssets, position: Vec3, state: &GameState) -> impl Bundle {
    let mut move_speed_mult = 1.0;
    let mut hop_speed_mult = 1.0;
    let mut time_between_hops = 0.2;
    let mut hop_time_length = 0.3;
    let mut jump_height_mult = 1.0;

    if state.is_modifier_active(Modifier::MoonGravity) {
        hop_speed_mult *= 0.8;
        // move_speed_mult *= 0.8;
        hop_time_length += 0.5;
        jump_height_mult *= 6.0;
    }
    if state.is_modifier_active(Modifier::HyperSheep) {
        hop_speed_mult *= 1.3;
        move_speed_mult *= 1.3;
        time_between_hops *= 0.1;
    }
    (
        Name::new("Sheep"),
        MovementController::new(move_speed_mult),
        HopMovementController {
            hop_speed_mult,
            time_between_hops,
            hop_time_length,
            jump_height_mult,
            ..Default::default()
        },
        Sheep::new()
            .default_speed_mult(move_speed_mult)
            .spooked_speed_mult(move_speed_mult * 2.0)
            .step_distance(move_speed_mult * 2.0),
        SceneRoot(sheep_assets.scene.clone()),
        Transform::from_translation(position),
        DespawnOnExit(Screen::Gameplay),
    )
}

fn sheep_wander(
    time: Res<Time>,
    mut sheep_query: Query<(&mut MovementController, &Transform, &mut Sheep)>,
) {
    for (mut movement, transform, mut sheep) in &mut sheep_query {
        if let SheepState::Wander(timer) = &mut sheep.state {
            timer.tick(time.delta());
            if timer.just_finished() {
                let rng = &mut rand::rng();
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let dir = Vec2::from_angle(angle);
                let target = transform.translation.xz() + dir * sheep.step_distance;
                movement.intent = target;
                sheep.reset_timer();
            }
        }
    }
}

fn sheep_state_update(
    time: Res<Time>,
    mut sheep_query: Query<(
        &mut MovementController,
        &mut HopMovementController,
        &Transform,
        &mut Sheep,
    )>,
    player_query: Query<&Transform, With<Player>>,
    goal_query: Query<&Transform, (With<GoalLocation>, Without<Player>)>,
    bounds: Res<LevelBounds>,
) {
    for (mut movement, mut controller, transform, mut sheep) in &mut sheep_query {
        let pos = transform.translation.xz();
        match sheep.state {
            SheepState::Wander(_) => {
                movement.move_speed_mult = sheep.default_speed_mult;
                for player_transform in player_query {
                    let player_pos = player_transform.translation.xz();
                    if pos.distance(player_pos) < SHEEP_INTERACT_RANGE {
                        sheep.state = SheepState::Evading(player_pos);
                    }
                }
            }
            SheepState::Evading(mut danger_pos) => {
                for player_transform in player_query {
                    let player_pos = player_transform.translation.xz();
                    if pos.distance(player_pos) < SHEEP_INTERACT_RANGE {
                        danger_pos = player_pos;
                    }
                }
                if pos.distance(danger_pos) >= SHEEP_INTERACT_RANGE {
                    sheep.state = SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                    sheep.reset_timer();
                } else {
                    let preferred = (pos - danger_pos).normalize_or(Vec2::X);
                    let dir = pick_evasion_dir(pos, preferred, &bounds);
                    movement.move_speed_mult = sheep.default_speed_mult;
                    movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                }
            }
            SheepState::Spooked(danger_pos) => {
                if pos.distance(danger_pos) >= SHEEP_INTERACT_RANGE * 2.0 {
                    sheep.state = SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                    sheep.reset_timer();
                } else {
                    let dir = (pos - danger_pos).normalize_or(Vec2::X);
                    movement.move_speed_mult = sheep.spooked_speed_mult;
                    movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                }
            }
            SheepState::BeingCounted => {
                let goal_pos = goal_query.single().unwrap().translation.xz();
                let dir = (goal_pos - pos).normalize_or(Vec2::X);
                controller.hop_speed_mult = 0.8;
                movement.move_speed_mult = 0.8;
                movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
            }
            SheepState::BeingAbducted => {
                movement.intent = transform.translation.xz();
            }
        }
    }
}

fn sheep_abduction_update(
    time: Res<Time>,
    mut commands: Commands,
    mut sheep_query: Query<(Entity, &mut Transform, &Sheep)>,
) {
    for (entity, mut transform, sheep) in &mut sheep_query {
        if !sheep.is_being_abducted() {
            continue;
        }

        transform.translation.y =
            (transform.translation.y + ABDUCTION_ASCENT_SPEED * time.delta_secs()).min(UFO_HEIGHT);

        if transform.translation.y >= UFO_HEIGHT - 2.0 {
            commands.entity(entity).despawn();
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
        match sheep.state {
            SheepState::BeingAbducted => {}
            SheepState::BeingCounted => {
                if pos.distance_squared(goal_pos) < 1.5 {
                    state.points += 1;
                    commands.entity(entity).despawn();
                }
            }
            _ => {
                if pos.distance_squared(goal_pos) < GOAL_RADIUS * GOAL_RADIUS {
                    sheep.state = SheepState::BeingCounted;
                }
            }
        }
    }
}

// To prevent sheep getting stuck in corners
fn pick_evasion_dir(pos: Vec2, preferred: Vec2, bounds: &LevelBounds) -> Vec2 {
    let candidates = [preferred.perp(), -preferred.perp(), -preferred];

    let mut best_dir = preferred;
    let target = bounds.clamp_to_bounds(pos + preferred);
    if target == pos + preferred {
        return preferred;
    }
    let mut best_score = target.distance_squared(pos);
    for dir in candidates {
        let target = bounds.clamp_to_bounds(pos + dir);
        let score = target.distance_squared(pos);
        if score > best_score {
            best_score = score;
            best_dir = dir;
        }
    }

    best_dir
}
