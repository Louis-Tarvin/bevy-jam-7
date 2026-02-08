//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use std::time::Duration;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_inspector_egui::egui::lerp;
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    audio::{sound_effect, sound_effect_3d},
    game::{level::LevelBounds, player::PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_hop_movement)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add)]
#[require(Transform)]
pub struct HopMovementController {
    /// Desired location on x,z plane
    pub intent: Vec2,
    pub current_hop_src: Option<Vec2>,
    pub current_hop_dest: Option<Vec2>,
    pub move_speed_mult: f32,
    pub hop_speed_mult: f32,
    pub time_between_hops: f32,
    pub hop_time_length: f32,
    pub airborne: bool,
    pub timer: Timer,
}

impl HopMovementController {
    pub fn new(
        move_speed_mult: f32,
        hop_speed_mult: f32,
        time_between_hops: f32,
        hop_time_length: f32,
    ) -> Self {
        Self {
            move_speed_mult,
            hop_speed_mult,
            time_between_hops,
            hop_time_length,
            ..Default::default()
        }
    }
}

impl Default for HopMovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            current_hop_src: None,
            current_hop_dest: None,
            move_speed_mult: 3.0,
            hop_speed_mult: 1.0,
            time_between_hops: 0.2,
            hop_time_length: 0.3,
            airborne: false,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

impl HopMovementController {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        // Initialise intent to the spawned position
        let pos = world
            .get::<Transform>(context.entity)
            .unwrap()
            .translation
            .xz();
        let mut entity = world.get_mut::<Self>(context.entity).unwrap();
        entity.intent = pos;
    }

    pub fn apply_movement(&mut self, direction: Vec2) {
        self.intent += direction * self.move_speed_mult;
    }

    /// Returns true if just started a hop
    pub fn update(&mut self, delta_secs: f32, current_pos: Vec2) -> bool {
        self.timer
            .tick(Duration::from_secs_f32(delta_secs * self.hop_speed_mult));
        if self.timer.is_finished() {
            if self.airborne {
                // Just hit the ground
                self.airborne = false;
                self.timer
                    .set_duration(Duration::from_secs_f32(self.time_between_hops));
                self.timer.reset();
            } else {
                // check that intent is sufficiently far to justify a hop
                if self.intent.distance_squared(current_pos) > 0.4 {
                    // Begin hop
                    self.airborne = true;
                    self.timer
                        .set_duration(Duration::from_secs_f32(self.hop_time_length));
                    self.timer.reset();
                    self.current_hop_src = Some(current_pos);
                    self.current_hop_dest = Some(self.intent);
                    return true;
                }
            }
        }
        false
    }
}

fn apply_hop_movement(
    time: Res<Time>,
    mut movement_query: Query<(&mut HopMovementController, &mut Transform)>,
    player_assets: If<Res<PlayerAssets>>,
    mut commands: Commands,
    bounds: Res<LevelBounds>,
) {
    for (mut controller, mut transform) in &mut movement_query {
        controller.intent = bounds.clamp_to_bounds(controller.intent);
        let just_hopped = controller.update(time.delta_secs(), transform.translation.xz());
        if controller.airborne {
            // Lerp from source to destination
            if let (Some(src), Some(dest)) =
                (controller.current_hop_src, controller.current_hop_dest)
            {
                let dest = bounds.clamp_to_bounds(dest);
                let x = lerp(src.x..=dest.x, controller.timer.fraction());
                let y = lerp(src.y..=dest.y, controller.timer.fraction());
                transform.translation.x = x;
                transform.translation.z = y;
                transform.translation.y = jump_height(controller.timer.fraction());
            }
        }
        if just_hopped {
            // rotate to face direction of movement
            if let (Some(src), Some(dest)) =
                (controller.current_hop_src, controller.current_hop_dest)
            {
                let dir = dest - src;
                if dir.length_squared() > 0.0001 {
                    let yaw = dir.x.atan2(dir.y);
                    transform.rotation = Quat::from_rotation_y(yaw);
                }
            }
            // play a random hop sound
            let rng = &mut rand::rng();
            let random_step = player_assets.steps.choose(rng).unwrap().clone();
            commands.spawn(sound_effect_3d(random_step, transform.translation));
        }
    }
}

fn jump_height(t: f32) -> f32 {
    4.0 * t * (1.0 - t)
}
