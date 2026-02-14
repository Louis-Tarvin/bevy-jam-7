//! Sheep behavior and spawning.

use std::time::Duration;

use bevy::{gltf::GltfMaterialName, math::ops::floor, prelude::*, scene::SceneInstanceReady};
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        level::{GOAL_RADIUS, GoalLocation, LevelBounds},
        modifiers::Modifier,
        movement::{HopMovementController, MovementController},
        player::Player,
        state::{GamePhase, GameState, shop::items::Charm},
        ufo::UFO_HEIGHT,
    },
    screens::Screen,
};

const ABDUCTION_ASCENT_SPEED: f32 = 6.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SheepAssets>();
    app.add_observer(apply_wool_material_on_scene_ready);
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

#[derive(Debug, Default, Reflect, Clone, PartialEq)]
pub enum SheepColor {
    #[default]
    White,
    Black,
    Blue,
    Red,
    Gold,
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Sheep {
    state: SheepState,
    color: SheepColor,
    step_distance: f32,
    min_wait: f32,
    max_wait: f32,
    default_speed_mult: f32,
    spooked_speed_mult: f32,
}

impl Sheep {
    fn new(color: SheepColor) -> Self {
        let mut sheep = Self {
            state: SheepState::Wander(Timer::from_seconds(1.0, TimerMode::Once)),
            color,
            step_distance: 1.5,
            min_wait: 1.5,
            max_wait: 5.0,
            default_speed_mult: 1.0,
            spooked_speed_mult: 1.7,
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
        if self.is_being_abducted() || matches!(self.state, SheepState::BeingCounted) {
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
    pub wool_white: Handle<StandardMaterial>,
    pub wool_black: Handle<StandardMaterial>,
    pub wool_blue: Handle<StandardMaterial>,
    pub wool_red: Handle<StandardMaterial>,
    pub wool_gold: Handle<StandardMaterial>,
}

impl FromWorld for SheepAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let scene = assets.load("obj/sheep.glb#Scene0");
        let mut mats = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            scene,
            wool_white: mats.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                perceptual_roughness: 0.9,
                ..Default::default()
            }),
            wool_black: mats.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.12),
                perceptual_roughness: 0.9,
                ..Default::default()
            }),
            wool_blue: mats.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 1.0),
                perceptual_roughness: 0.9,
                ..Default::default()
            }),
            wool_red: mats.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.3, 0.3),
                perceptual_roughness: 0.9,
                ..Default::default()
            }),
            wool_gold: mats.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.82, 0.2),
                perceptual_roughness: 0.5,
                metallic: 0.6,
                ..Default::default()
            }),
        }
    }
}

pub fn sheep(
    sheep_assets: &SheepAssets,
    position: Vec3,
    state: &GameState,
    color: SheepColor,
) -> impl Bundle {
    let color = if matches!(color, SheepColor::White) && rand::rng().random_bool(0.05) {
        SheepColor::Black
    } else {
        color
    };

    let mut move_speed_mult = 2.0;
    let mut hop_speed_mult = 2.5;
    let mut time_between_hops = 0.2;
    let mut hop_time_length = 0.3;
    let mut jump_height_mult = 1.0;

    if state.is_modifier_active(Modifier::MoonGravity) {
        hop_speed_mult *= 0.5;
        // move_speed_mult *= 0.8;
        hop_time_length += 0.5;
        jump_height_mult *= 6.0;
    }
    if state.is_modifier_active(Modifier::HyperSheep) {
        hop_speed_mult *= 2.0;
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
        Sheep::new(color)
            .default_speed_mult(move_speed_mult)
            .spooked_speed_mult(move_speed_mult * 2.0)
            .step_distance(move_speed_mult),
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
    player_query: Query<(&Transform, &Player)>,
    goal_query: Query<&Transform, (With<GoalLocation>, Without<Player>)>,
    bounds: Res<LevelBounds>,
    game_state: Res<GameState>,
) {
    for (mut movement, mut controller, transform, mut sheep) in &mut sheep_query {
        let pos = transform.translation.xz();
        match sheep.state {
            SheepState::Wander(_) => {
                movement.move_speed_mult = sheep.default_speed_mult;
                for (player_transform, player) in player_query {
                    let player_pos = player_transform.translation.xz();
                    if pos.distance(player_pos) < player.sheep_interact_radius {
                        sheep.state = SheepState::Evading(player_pos);
                    }
                }
            }
            SheepState::Evading(mut danger_pos) => {
                for (player_transform, player) in player_query {
                    let player_pos = player_transform.translation.xz();
                    if pos.distance(player_pos) < player.sheep_interact_radius {
                        danger_pos = player_pos;
                    }
                    if pos.distance(danger_pos) >= player.sheep_interact_radius {
                        sheep.state = SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                        sheep.reset_timer();
                    } else {
                        let preferred = (pos - danger_pos).normalize_or(Vec2::X);
                        let dir = pick_evasion_dir(pos, preferred, &bounds);
                        movement.move_speed_mult = sheep.default_speed_mult;
                        movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                    }
                }
            }
            SheepState::Spooked(danger_pos) => {
                for (_, player) in player_query {
                    if game_state.is_charm_active(Charm::WellTrained) {
                        if pos.distance(danger_pos) < player.sheep_interact_radius {
                            sheep.state =
                                SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                            sheep.reset_timer();
                        } else {
                            let dir = (danger_pos - pos).normalize_or(Vec2::X);
                            movement.move_speed_mult = sheep.default_speed_mult;
                            movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                        }
                    } else {
                        if pos.distance(danger_pos) >= player.sheep_interact_radius + 8.0 {
                            sheep.state =
                                SheepState::Wander(Timer::from_seconds(0.5, TimerMode::Once));
                            sheep.reset_timer();
                        } else {
                            let dir = (pos - danger_pos).normalize_or(Vec2::X);
                            movement.move_speed_mult = sheep.spooked_speed_mult;
                            movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
                        }
                    }
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
    sheep_assets: Res<SheepAssets>,
    bounds: Res<LevelBounds>,
) {
    let goal_pos = goal_query.translation.xz();
    for (entity, sheep_transform, mut sheep_c) in sheep_query {
        let pos = sheep_transform.translation.xz();
        match sheep_c.state {
            SheepState::BeingAbducted => {}
            SheepState::BeingCounted => {
                if pos.distance_squared(goal_pos) < 1.5 {
                    match sheep_c.color {
                        SheepColor::White => {
                            state.points += 1;
                        }
                        SheepColor::Blue => {
                            state.points += 5;
                        }
                        SheepColor::Red => {
                            state.points = floor(state.points as f32 * 1.5) as u32;
                        }
                        SheepColor::Black => {
                            state.points += 1;
                            if state.is_charm_active(Charm::Exponential) {
                                let rng = &mut rand::rng();
                                let x = rng.random_range(bounds.min.x..=bounds.max.x);
                                let z = rng.random_range(bounds.min.y..=bounds.max.y);
                                let pos = Vec3::new(x, 0.0, z);
                                commands.spawn((
                                    sheep(&sheep_assets, pos, &state, SheepColor::Black),
                                    DespawnOnExit(GamePhase::Herding),
                                ));
                                let x = rng.random_range(bounds.min.x..=bounds.max.x);
                                let z = rng.random_range(bounds.min.y..=bounds.max.y);
                                let pos = Vec3::new(x, 0.0, z);
                                commands.spawn((
                                    sheep(&sheep_assets, pos, &state, SheepColor::Black),
                                    DespawnOnExit(GamePhase::Herding),
                                ));
                            }
                        }
                        SheepColor::Gold => {
                            state.money += 1;
                        }
                    }
                    commands.entity(entity).despawn();
                }
            }
            _ => {
                if pos.distance_squared(goal_pos) < GOAL_RADIUS * GOAL_RADIUS {
                    sheep_c.state = SheepState::BeingCounted;
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

fn apply_wool_material_on_scene_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    sheep_q: Query<&Sheep>,
    children: Query<&Children>,
    mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(sheep) = sheep_q.get(scene_ready.entity) else {
        return;
    };

    let tint = match sheep.color {
        SheepColor::White => Color::srgb(1.0, 1.0, 1.0),
        SheepColor::Black => Color::srgb(0.12, 0.12, 0.12),
        SheepColor::Blue => Color::srgb(0.3, 0.5, 1.0),
        SheepColor::Red => Color::srgb(1.0, 0.3, 0.3),
        SheepColor::Gold => Color::srgb(1.0, 0.82, 0.2),
    };

    for descendant in children.iter_descendants(scene_ready.entity) {
        let Ok((mat_handle, mat_name)) = mesh_materials.get(descendant) else {
            continue;
        };

        if mat_name.0 != "wool" {
            continue;
        }

        let Some(mut new_mat) = materials.get(mat_handle.id()).cloned() else {
            continue;
        };
        new_mat.base_color = tint;

        commands
            .entity(descendant)
            .insert(MeshMaterial3d(materials.add(new_mat)));
    }
}
