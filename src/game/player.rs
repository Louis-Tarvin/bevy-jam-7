//! Player-specific behavior.

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{movement::HopMovementController, sheep::Sheep},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    app.add_systems(
        Update,
        (record_player_directional_input, handle_bark)
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        tick_player_timers
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    (
        Name::new("Player"),
        Player::default(),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        HopMovementController::new(3.0, 1.0, 0.1, 0.2),
        SpatialListener::new(0.2),
    )
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub bark_radius: f32,
    pub bark_cooldown: Timer,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            bark_radius: 8.0,
            bark_cooldown: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

fn tick_player_timers(time: Res<Time>, player_query: Query<&mut Player>) {
    for mut player in player_query {
        player.bark_cooldown.tick(time.delta());
    }
}

fn handle_bark(
    player_query: Query<(&mut Player, &Transform)>,
    mut sheep_query: Query<(&mut Sheep, &Transform), Without<Player>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyE) || input.just_pressed(KeyCode::Space) {
        for (mut player, player_transform) in player_query {
            if player.bark_cooldown.is_finished() {
                let player_pos = player_transform.translation.xz();
                player.bark_cooldown.reset();
                for (mut sheep, sheep_transform) in sheep_query.iter_mut() {
                    let sheep_pos = sheep_transform.translation.xz();
                    if player_pos.distance_squared(sheep_pos)
                        <= player.bark_radius * player.bark_radius
                    {
                        sheep.become_spooked(player_pos);
                    }
                }
            }
        }
    }
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut controller_query: Query<&mut HopMovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        let speed_mult = controller.move_speed_mult;
        controller.apply_movement(intent * speed_mult * time.delta_secs());
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
