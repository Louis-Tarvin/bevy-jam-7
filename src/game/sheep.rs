//! Sheep behavior and spawning.

use std::{collections::HashMap, time::Duration};

use bevy::{gltf::GltfMaterialName, math::ops::floor, prelude::*, scene::SceneInstanceReady};
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        level::{GOAL_RADIUS, GoalLocation, GoalTextMessage, LevelBounds},
        modifiers::Modifier,
        movement::{HopMovementController, MovementController},
        player::Player,
        state::{GamePhase, GameState, RoundStats, shop::items::Charm},
        ufo::UFO_HEIGHT,
    },
    screens::Screen,
};

const ABDUCTION_ASCENT_SPEED: f32 = 6.0;
const HERD_RADIUS: f32 = 10.0;
const HERD_RADIUS_SQ: f32 = HERD_RADIUS * HERD_RADIUS;
const HERD_SEPARATION_RADIUS: f32 = 2.4;
const HERD_SEPARATION_RADIUS_SQ: f32 = HERD_SEPARATION_RADIUS * HERD_SEPARATION_RADIUS;
const HERD_CELL_SIZE: f32 = HERD_RADIUS;
const HERD_COHESION_WEIGHT: f32 = 0.9;
const HERD_SEPARATION_WEIGHT: f32 = 1.5;
const HERD_EVADE_BLEND: f32 = 0.55;
const HERD_WANDER_JITTER: f32 = 0.35;
const HERD_UPDATE_INTERVAL_SECS: f32 = 0.10;
const HERD_UPDATE_BUCKETS: u64 = 4;
const HERD_MAX_NEIGHBORS: usize = 20;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SheepAssets>();
    app.add_observer(apply_wool_material_on_scene_ready);
    app.add_systems(
        Update,
        (
            sheep_goal_check,
            sheep_state_update,
            (sheep_wander, sheep_herding, sheep_abduction_update),
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
    herd_dir: Vec2,
}

impl Sheep {
    fn new(color: SheepColor) -> Self {
        let mut sheep = Self {
            state: SheepState::Wander(Timer::from_seconds(1.0, TimerMode::Once)),
            color,
            step_distance: 1.5,
            min_wait: 1.5,
            max_wait: 7.0,
            default_speed_mult: 1.2,
            spooked_speed_mult: 1.9,
            herd_dir: Vec2::ZERO,
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
    let p = if state.is_charm_active(Charm::Ink) {
        0.1
    } else {
        0.05
    };
    let color = if matches!(color, SheepColor::White) && rand::rng().random_bool(p) {
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
    bounds: Res<LevelBounds>,
    mut sheep_query: Query<(&mut MovementController, &Transform, &mut Sheep)>,
) {
    for (mut movement, transform, mut sheep) in &mut sheep_query {
        if let SheepState::Wander(timer) = &mut sheep.state {
            timer.tick(time.delta());
            if timer.just_finished() {
                let rng = &mut rand::rng();
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let random_dir = Vec2::from_angle(angle);
                let herd_dir = sheep.herd_dir;
                let dir = if herd_dir == Vec2::ZERO {
                    random_dir
                } else {
                    (herd_dir + random_dir * HERD_WANDER_JITTER).normalize_or(random_dir)
                };
                let target =
                    bounds.clamp_to_bounds(transform.translation.xz() + dir * sheep.step_distance);
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
                        let steer = (dir + sheep.herd_dir * HERD_EVADE_BLEND).normalize_or(dir);
                        movement.move_speed_mult = sheep.default_speed_mult;
                        movement.apply_movement(steer * time.delta_secs() * sheep.step_distance);
                    }
                }
            }
            SheepState::Spooked(danger_pos) => {
                sheep.herd_dir = Vec2::ZERO;
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
                sheep.herd_dir = Vec2::ZERO;
                let goal_pos = goal_query.single().unwrap().translation.xz();
                let dir = (goal_pos - pos).normalize_or(Vec2::X);
                controller.hop_speed_mult = 0.8;
                movement.move_speed_mult = 0.8;
                movement.apply_movement(dir * time.delta_secs() * sheep.step_distance);
            }
            SheepState::BeingAbducted => {
                sheep.herd_dir = Vec2::ZERO;
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

fn sheep_herding(
    time: Res<Time>,
    mut herd_timer: Local<Timer>,
    mut herd_bucket: Local<u64>,
    mut set: ParamSet<(
        Query<(Entity, &Transform, &Sheep)>,
        Query<(Entity, &Transform, &mut Sheep)>,
    )>,
) {
    if herd_timer.duration().is_zero() {
        *herd_timer = Timer::from_seconds(HERD_UPDATE_INTERVAL_SECS, TimerMode::Repeating);
    }
    if !herd_timer.tick(time.delta()).just_finished() {
        return;
    }

    *herd_bucket = (*herd_bucket + 1) % HERD_UPDATE_BUCKETS;
    let active_bucket = *herd_bucket;

    let snapshot: Vec<(Entity, Vec2)> = set
        .p0()
        .iter()
        .filter(|(_, _, sheep)| {
            matches!(sheep.state, SheepState::Wander(_) | SheepState::Evading(_))
        })
        .map(|(entity, transform, _)| (entity, transform.translation.xz()))
        .collect();
    if snapshot.len() < 2 {
        return;
    }

    let mut grid: HashMap<IVec2, Vec<usize>> = HashMap::default();
    for (index, (_, position)) in snapshot.iter().enumerate() {
        grid.entry(spatial_cell(*position)).or_default().push(index);
    }

    for (entity, transform, mut sheep) in &mut set.p1() {
        if !matches!(sheep.state, SheepState::Wander(_) | SheepState::Evading(_)) {
            continue;
        }
        if entity.to_bits() % HERD_UPDATE_BUCKETS != active_bucket {
            continue;
        }

        let pos = transform.translation.xz();
        let cell = spatial_cell(pos);
        let mut center = Vec2::ZERO;
        let mut nearby_count = 0.0;
        let mut separation = Vec2::ZERO;
        let mut sampled_neighbors = 0usize;

        'neighbor_cells: for dy in -1..=1 {
            for dx in -1..=1 {
                let neighbor_cell = IVec2::new(cell.x + dx, cell.y + dy);
                let Some(indices) = grid.get(&neighbor_cell) else {
                    continue;
                };

                for &index in indices {
                    let (other_entity, other_pos): (Entity, Vec2) = snapshot[index];
                    if other_entity == entity {
                        continue;
                    }

                    let offset = other_pos - pos;
                    let dist_sq = offset.length_squared();
                    if dist_sq > HERD_RADIUS_SQ {
                        continue;
                    }

                    center += other_pos;
                    nearby_count += 1.0;
                    sampled_neighbors += 1;

                    if dist_sq > 0.0 && dist_sq < HERD_SEPARATION_RADIUS_SQ {
                        let dist = dist_sq.sqrt();
                        let push_strength =
                            (HERD_SEPARATION_RADIUS - dist) / HERD_SEPARATION_RADIUS;
                        separation += (pos - other_pos).normalize_or(Vec2::X) * push_strength;
                    }

                    if sampled_neighbors >= HERD_MAX_NEIGHBORS {
                        break 'neighbor_cells;
                    }
                }
            }
        }

        if nearby_count <= 0.0 {
            sheep.herd_dir = Vec2::ZERO;
            continue;
        }

        let cohesion = ((center / nearby_count) - pos).normalize_or_zero() * HERD_COHESION_WEIGHT;
        let avoid = separation.normalize_or_zero() * HERD_SEPARATION_WEIGHT;
        sheep.herd_dir = (cohesion + avoid).normalize_or_zero();
    }
}

fn sheep_goal_check(
    mut commands: Commands,
    sheep_query: Query<(Entity, &Transform, &mut Sheep)>,
    goal_query: Single<&Transform, With<GoalLocation>>,
    mut state: ResMut<GameState>,
    mut round_stats: ResMut<RoundStats>,
    sheep_assets: Res<SheepAssets>,
    bounds: Res<LevelBounds>,
    mut writer: MessageWriter<GoalTextMessage>,
) {
    let goal_pos = goal_query.translation.xz();
    for (entity, sheep_transform, mut sheep_c) in sheep_query {
        let pos = sheep_transform.translation.xz();
        match sheep_c.state {
            SheepState::BeingAbducted => {}
            SheepState::BeingCounted => {
                if pos.distance_squared(goal_pos) < 1.5 {
                    let is_first_counted = round_stats.sheep_counted == 0;
                    if is_first_counted && state.is_charm_active(Charm::Cloning) {
                        state.sheep_count += 1;
                        match sheep_c.color {
                            SheepColor::Blue => state.blue_sheep_count += 1,
                            SheepColor::Red => state.red_sheep_count += 1,
                            SheepColor::Black => state.black_sheep_count += 1,
                            SheepColor::Gold => state.gold_sheep_count += 1,
                            _ => {}
                        }
                        writer.write(GoalTextMessage {
                            text: "Cloned".to_string(),
                            color: Some(Color::srgb(0.55, 0.85, 0.95)),
                        });
                    }

                    match sheep_c.color {
                        SheepColor::White => {
                            if state.is_charm_active(Charm::Evolution) {
                                round_stats.white_sheep_counted += 1;
                                if round_stats.white_sheep_counted % 5 == 0 {
                                    state.blue_sheep_count += 1;
                                    writer.write(GoalTextMessage {
                                        text: "Evolved to blue".to_string(),
                                        color: Some(Color::srgb(0.3, 0.4, 0.8)),
                                    });
                                } else {
                                    writer.write(GoalTextMessage {
                                        text: "0 points".to_string(),
                                        color: None,
                                    });
                                }
                            } else {
                                state.points += 1;
                                writer.write(GoalTextMessage {
                                    text: "+1 point".to_string(),
                                    color: None,
                                });
                            }
                        }
                        SheepColor::Blue => {
                            state.points += 5;
                            writer.write(GoalTextMessage {
                                text: "+5 points".to_string(),
                                color: Some(Color::srgb(0.3, 0.4, 0.8)),
                            });
                        }
                        SheepColor::Red => {
                            if is_first_counted && state.is_charm_active(Charm::RedToGold) {
                                state.red_sheep_count -= 1;
                                state.gold_sheep_count += 1;
                            }
                            state.points = floor(state.points as f32 * 1.5) as u32;
                            writer.write(GoalTextMessage {
                                text: "points x1.5".to_string(),
                                color: Some(Color::srgb(1.0, 0.3, 0.3)),
                            });
                        }
                        SheepColor::Black => {
                            round_stats.black_sheep_counted += 1;
                            state.points += 1;
                            writer.write(GoalTextMessage {
                                text: "+1 point".to_string(),
                                color: None,
                            });
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
                            writer.write(GoalTextMessage {
                                text: "+1 gold".to_string(),
                                color: Some(Color::srgb(1.0, 0.82, 0.2)),
                            });
                        }
                    }
                    round_stats.sheep_counted += 1;
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

fn spatial_cell(position: Vec2) -> IVec2 {
    IVec2::new(
        (position.x / HERD_CELL_SIZE).floor() as i32,
        (position.y / HERD_CELL_SIZE).floor() as i32,
    )
}

fn apply_wool_material_on_scene_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    sheep_q: Query<&Sheep>,
    children: Query<&Children>,
    mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
    sheep_assets: Res<SheepAssets>,
) {
    let Ok(sheep) = sheep_q.get(scene_ready.entity) else {
        return;
    };

    let material = match sheep.color {
        SheepColor::White => sheep_assets.wool_white.clone(),
        SheepColor::Black => sheep_assets.wool_black.clone(),
        SheepColor::Blue => sheep_assets.wool_blue.clone(),
        SheepColor::Red => sheep_assets.wool_red.clone(),
        SheepColor::Gold => sheep_assets.wool_gold.clone(),
    };

    for descendant in children.iter_descendants(scene_ready.entity) {
        let Ok((_mat_handle, mat_name)) = mesh_materials.get(descendant) else {
            continue;
        };

        if mat_name.0 != "wool" {
            continue;
        }

        commands
            .entity(descendant)
            .insert(MeshMaterial3d(material.clone()));
    }
}
