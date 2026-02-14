use bevy::prelude::*;

use crate::{
    game::{
        modifiers::Modifier,
        state::{GamePhase, GameState, NewRoundInfo},
    },
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePhase::ModifierChoice), on_modifier_choice);
}

fn on_modifier_choice(mut commands: Commands, mut game_state: ResMut<GameState>) {
    let NewRoundInfo {
        removed_modifier,
        modifier_choices,
    } = game_state.new_round();

    draw_choice_ui(&mut commands, removed_modifier, &modifier_choices);
}

fn draw_choice_ui(
    commands: &mut Commands,
    removed_modifier: Option<Modifier>,
    modifier_choices: &[Modifier],
) {
    commands
        .spawn((
            widget::ui_root("Modifier choice UI"),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            GlobalZIndex(2),
            DespawnOnExit(GamePhase::ModifierChoice),
        ))
        .with_children(|root| {
            root.spawn(widget::panel()).with_children(|panel| {
                panel.spawn(widget::header("Choose new dream modifier:"));
                if let Some(removed_modifier) = removed_modifier {
                    panel.spawn(widget::label(format!(
                        "Modifier no longer active: {}",
                        removed_modifier.name()
                    )));
                }
                panel
                    .spawn((
                        Name::new("Modifiers Row"),
                        Node {
                            width: percent(100),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::FlexStart,
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(24),
                            row_gap: px(24),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        for choice in modifier_choices {
                            row.spawn(modifier_card(*choice));
                        }
                    });
            });
        });
}

fn modifier_card(modifier: Modifier) -> impl Bundle {
    (
        Name::new(format!("Modifier Card {}", modifier.name())),
        Node {
            width: px(350),
            max_width: percent(100),
            min_height: px(300),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
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
            (
                Name::new("Modifier Value"),
                Text(format!("+{} money", modifier.difficulty().coins_given())),
                TextFont::from_font_size(14.0),
                TextColor(ui_palette::LABEL_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ),
            widget::button_medium(
                "Choose",
                move |_: On<Pointer<Click>>,
                      mut next_state: ResMut<NextState<GamePhase>>,
                      mut state: ResMut<GameState>| {
                    state.active_modifiers.push(modifier);
                    state.money += modifier.difficulty().coins_given() as u32;
                    next_state.set(GamePhase::Shop);
                }
            )
        ],
    )
}
