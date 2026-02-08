use bevy::prelude::*;

use crate::{
    game::state::{GamePhase, GameState},
    theme::prelude::*,
};

pub fn draw_herding_ui(commands: &mut Commands) {
    commands.spawn((
        widget::ui_root("root"),
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
