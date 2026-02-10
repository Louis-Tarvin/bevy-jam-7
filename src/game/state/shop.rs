use bevy::prelude::*;

use crate::{
    game::{
        modifiers::Modifier,
        state::{GamePhase, GameState},
    },
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePhase::Shop), on_shop);
    app.add_systems(
        Update,
        update_shop_ui
            .run_if(in_state(GamePhase::Shop))
            .run_if(resource_changed::<GameState>),
    );
}

fn on_shop(mut commands: Commands, game_state: Res<GameState>) {
    let active_modifiers = game_state.active_modifiers.clone();
    let money = game_state.money;
    let point_target = game_state.point_target;
    commands.spawn((
        widget::ui_root("Shop UI"),
        DespawnOnExit(GamePhase::Shop),
        children![(
            widget::panel(),
            children![
                widget::header("Active Modifiers"),
                (
                    widget::row(),
                    Children::spawn(SpawnIter(active_modifiers.into_iter().map(modifier_card)))
                ),
                widget::divider(),
                widget::header("Shop"),
                (widget::label(format!("Money: {}", money)), ShopMoneyText),
                widget::button("Buy Sheep (1)", buy_sheep),
                widget::divider(),
                widget::header("Next Round"),
                (
                    widget::label(format!("Points target: {}", point_target)),
                    ShopTargetText,
                ),
                widget::button("Start", start_next_round)
            ]
        )],
    ));
}

fn modifier_card(modifier: Modifier) -> impl Bundle {
    (
        Name::new(format!("Modifier Card {}", modifier.name())),
        Node {
            width: px(250),
            max_width: percent(100),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.95)),
        children![
            (
                Name::new("Modifier Name"),
                Text(modifier.name().to_string()),
                TextFont::from_font_size(22.0),
                TextColor(ui_palette::HEADER_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ),
            (
                Name::new("Modifier Description"),
                Text(modifier.description().to_string()),
                TextFont::from_font_size(16.0),
                TextColor(ui_palette::LABEL_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ),
        ],
    )
}

fn start_next_round(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<GamePhase>>) {
    next_state.set(GamePhase::Herding);
}

fn buy_sheep(_: On<Pointer<Click>>, mut game_state: ResMut<GameState>) {
    if game_state.money > 0 {
        game_state.money -= 1;
        game_state.sheep_count += 1;
    }
}

#[derive(Component)]
struct ShopMoneyText;

#[derive(Component)]
struct ShopTargetText;

fn update_shop_ui(
    state: Res<GameState>,
    mut labels: ParamSet<(
        Single<&mut Text, With<ShopMoneyText>>,
        Single<&mut Text, With<ShopTargetText>>,
    )>,
) {
    labels.p0().0 = format!("Money: {}", state.money);
    labels.p1().0 = format!("Points target: {}", state.point_target);
}
