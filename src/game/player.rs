//! Player-specific behavior.

use bevy::{light::NotShadowCaster, prelude::*};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    game::{
        level::RandomTeleport,
        modifiers::Modifier,
        movement::MovementController,
        sheep::Sheep,
        state::{GamePhase, GameState},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    app.add_systems(
        Update,
        (record_player_directional_input, handle_bark)
            .run_if(in_state(GamePhase::Herding))
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        tick_player_timers
            .run_if(in_state(GamePhase::Herding))
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        init_player_gear_visuals
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(player_assets: &PlayerAssets, bark_radius: f32, is_sphere: bool) -> impl Bundle {
    let scene = if is_sphere {
        player_assets.scene_sphere.clone()
    } else {
        player_assets.scene.clone()
    };
    (
        Name::new("Player"),
        Player::new(bark_radius),
        SceneRoot(scene),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MovementController::new(3.0),
        SpatialListener::new(0.2),
    )
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub bark_radius: f32,
    pub sheep_interact_radius: f32,
    pub bark_cooldown: Timer,
}
impl Player {
    pub fn new(bark_radius: f32) -> Self {
        Self {
            bark_radius,
            sheep_interact_radius: 7.0,
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
    player_query: Query<(Entity, &mut Player, &Transform)>,
    mut sheep_query: Query<(Entity, &mut Sheep, &Transform), Without<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    game_state: Res<GameState>,
    assets: Res<PlayerAssets>,
) {
    if input.just_pressed(KeyCode::KeyE) || input.just_pressed(KeyCode::Space) {
        for (entity, mut player, player_transform) in player_query {
            if player.bark_cooldown.is_finished() {
                let player_pos = player_transform.translation.xz();
                player.bark_cooldown.reset();
                for (sheep_entity, mut sheep, sheep_transform) in sheep_query.iter_mut() {
                    let sheep_pos = sheep_transform.translation.xz();
                    if player_pos.distance_squared(sheep_pos)
                        <= player.bark_radius * player.bark_radius
                    {
                        if game_state.is_modifier_active(Modifier::SheepTeleport) {
                            commands.trigger(RandomTeleport {
                                entity: sheep_entity,
                            });
                        } else {
                            sheep.become_spooked(player_pos);
                        }
                    }
                }
                commands.spawn(sound_effect(assets.bark.clone()));
                if game_state.is_modifier_active(Modifier::TeleportingBark) {
                    commands.trigger(RandomTeleport { entity });
                }
            }
        }
    }
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
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

fn init_player_gear_visuals(
    game_state: Res<GameState>,
    mut commands: Commands,
    gear_query: Query<(Entity, &Name), (Without<Player>, Added<Name>)>,
    parent_query: Query<&ChildOf>,
    player_query: Query<(), With<Player>>,
) {
    let show_gear = game_state.is_modifier_active(Modifier::Space);

    for (entity, name) in &gear_query {
        let is_gear = name.contains("Helmet") || name.contains("Jetpack");
        if !is_gear || !is_descendant_of_player(entity, &parent_query, &player_query) {
            continue;
        }

        if show_gear {
            commands.entity(entity).insert(Visibility::Visible);
        } else {
            commands.entity(entity).insert(Visibility::Hidden);
        }

        if name.as_str() == "Helmet" {
            commands.entity(entity).insert(NotShadowCaster);
        }
    }
}

fn is_descendant_of_player(
    mut entity: Entity,
    parent_query: &Query<&ChildOf>,
    player_query: &Query<(), With<Player>>,
) -> bool {
    loop {
        if player_query.get(entity).is_ok() {
            return true;
        }

        let Ok(parent) = parent_query.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
    #[dependency]
    pub bark: Handle<AudioSource>,
    #[dependency]
    pub scene: Handle<Scene>,
    #[dependency]
    pub scene_sphere: Handle<Scene>,
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
            bark: assets.load("audio/sound_effects/bark.ogg"),
            scene: assets.load("obj/dog.glb#Scene0"),
            scene_sphere: assets.load("obj/dog.glb#Scene1"),
        }
    }
}
