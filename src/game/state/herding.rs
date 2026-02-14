use bevy::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    game::{
        camera::CameraTarget,
        level::LevelBounds,
        modifiers::Modifier,
        movement::{HopMovementController, SpaceMovementController},
        player::{PlayerAssets, player},
        sheep::{SheepAssets, SheepColor, sheep},
        state::{GamePhase, GameState, RoundStats, shop::items::Charm},
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
    mut round_stats: ResMut<RoundStats>,
    bounds: Res<LevelBounds>,
    mut camera_target: ResMut<CameraTarget>,
) {
    *round_stats = RoundStats::default();

    let total_sheep = game_state.sheep_count as usize;
    if total_sheep == 0 {
        return;
    }

    let mut sheep_colors = build_sheep_colors(&game_state);
    let rng = &mut rand::rng();

    if game_state.is_charm_active(Charm::GoldenSheep) {
        sheep_colors.push(SheepColor::Gold);
    }

    // spawn sheep
    for color in sheep_colors {
        let x = rng.random_range(bounds.min.x..=bounds.max.x);
        let z = rng.random_range(bounds.min.y..=bounds.max.y);
        let pos = Vec3::new(x, 0.0, z);
        commands.spawn((
            sheep(&sheep_assets, pos, &game_state, color),
            DespawnOnExit(GamePhase::Herding),
        ));
    }

    // spawn player
    let player = commands
        .spawn((
            player(&player_assets, game_state.player_bark_radius),
            DespawnOnExit(GamePhase::Herding),
        ))
        .id();
    if game_state.is_modifier_active(Modifier::Space) {
        commands
            .entity(player)
            .insert(SpaceMovementController::new(10.0));
    } else {
        commands
            .entity(player)
            .insert(HopMovementController::new(1.2, 0.1, 0.2));
    }
    camera_target.0 = Some(player);

    draw_herding_ui(&mut commands);
}

fn build_sheep_colors(game_state: &GameState) -> Vec<SheepColor> {
    let total_sheep = if game_state.is_charm_active(Charm::HalfTimeDoubleSheep) {
        game_state.sheep_count as usize * 2
    } else {
        game_state.sheep_count as usize
    };
    let mut colors = Vec::with_capacity(total_sheep);

    let colored_counts = [
        (SheepColor::Blue, game_state.blue_sheep_count as usize),
        (SheepColor::Red, game_state.red_sheep_count as usize),
        (SheepColor::Black, game_state.black_sheep_count as usize),
        (SheepColor::Gold, game_state.gold_sheep_count as usize),
    ];

    for (color, mut count) in colored_counts {
        if game_state.is_charm_active(Charm::HalfTimeDoubleSheep) {
            count *= 2;
        }
        colors.extend(std::iter::repeat_n(color, count));
    }

    if colors.len() > total_sheep {
        colors.truncate(total_sheep);
    }

    let white_count = total_sheep.saturating_sub(colors.len());
    colors.extend(std::iter::repeat_n(SheepColor::White, white_count));
    colors
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
