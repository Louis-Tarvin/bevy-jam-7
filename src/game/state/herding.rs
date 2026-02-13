use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        camera::CameraTarget,
        modifiers::Modifier,
        movement::{HopMovementController, SpaceMovementController},
        player::{PlayerAssets, player},
        sheep::{SheepAssets, sheep},
        state::{GamePhase, GameState},
    },
    screens::Screen,
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePhase::Herding), on_herding);
    app.add_systems(
        Update,
        tick_countdown
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems)
            .run_if(in_state(GamePhase::Herding)),
    );
    app.add_systems(
        Update,
        update_herding_ui.run_if(in_state(GamePhase::Herding)),
    );
    app.add_systems(
        Update,
        check_points_goal.run_if(resource_changed::<GameState>),
    );
}

pub fn tick_countdown(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    state.countdown.tick(time.delta());
    if state.countdown.just_finished() {
        next_state.set(Screen::Title);
    }
}

pub fn on_herding(
    mut commands: Commands,
    sheep_assets: Res<SheepAssets>,
    player_assets: Res<PlayerAssets>,
    game_state: Res<GameState>,
    mut camera_target: ResMut<CameraTarget>,
) {
    let count = game_state.sheep_count as usize;
    if count == 0 {
        return;
    }

    let grid = (count as f32).sqrt().ceil() as usize;
    let spacing = 10.0;
    let offset = (grid as f32 - 1.0) * 0.5;

    // spawn sheep
    for i in 0..count {
        let x = (i % grid) as f32;
        let z = (i / grid) as f32;
        let pos = Vec3::new((x - offset) * spacing, 0.0, (z - offset) * spacing);
        commands.spawn((
            sheep(&sheep_assets, pos, &game_state),
            DespawnOnExit(GamePhase::Herding),
        ));
    }

    // spawn player
    let player = commands
        .spawn((player(&player_assets), DespawnOnExit(GamePhase::Herding)))
        .id();
    if game_state.is_modifier_active(Modifier::Space) {
        commands
            .entity(player)
            .insert(SpaceMovementController::new(20.0));
    } else {
        commands
            .entity(player)
            .insert(HopMovementController::new(1.0, 0.1, 0.2));
    }
    camera_target.0 = Some(player);

    draw_herding_ui(&mut commands);
}

fn check_points_goal(game_state: Res<GameState>, mut next_state: ResMut<NextState<GamePhase>>) {
    if game_state.points >= game_state.point_target {
        next_state.set(GamePhase::ModifierChoice);
    }
}

fn draw_herding_ui(commands: &mut Commands) {
    commands.spawn((
        widget::ui_root("Herding UI root"),
        DespawnOnExit(GamePhase::Herding),
        children![(
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(16)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Pickable::IGNORE,
            children![
                (widget::label("Time: 0"), HerdingTimerText),
                (
                    Node {
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6),
                        ..default()
                    },
                    children![
                        (widget::label("Points: 0"), HerdingPointsText),
                        (widget::label("Target: 0"), HerdingTargetText),
                    ],
                ),
            ],
        ),],
    ));
}

#[derive(Component)]
pub struct HerdingTimerText;

#[derive(Component)]
pub struct HerdingPointsText;

#[derive(Component)]
pub struct HerdingTargetText;

pub fn update_herding_ui(
    state: Res<GameState>,
    mut labels: ParamSet<(
        Single<&mut Text, With<HerdingTimerText>>,
        Single<&mut Text, With<HerdingPointsText>>,
        Single<&mut Text, With<HerdingTargetText>>,
    )>,
) {
    let remaining = state
        .countdown
        .duration()
        .saturating_sub(state.countdown.elapsed());
    let seconds = remaining.as_secs_f32().ceil().max(0.0) as u32;
    labels.p0().0 = format!("Time: {seconds}");
    labels.p1().0 = format!("Points: {}", state.points);
    labels.p2().0 = format!("Target: {}", state.point_target);
}
