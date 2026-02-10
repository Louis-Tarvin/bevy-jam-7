use bevy::{prelude::*, text::TextBounds};

use crate::{
    game::{
        modifiers::Modifier,
        state::{GamePhase, GameState},
    },
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePhase::Interlude), on_interlude);
}

fn on_interlude(mut commands: Commands, mut game_state: ResMut<GameState>) {
    let removed_modifier = game_state.new_round();
    let active_modifiers = game_state.active_modifiers.as_slice();

    draw_interlude_ui(
        &mut commands,
        removed_modifier,
        active_modifiers,
        game_state.point_target,
    );
}

fn draw_interlude_ui(
    commands: &mut Commands,
    removed_modifier: Option<Modifier>,
    active_modifiers: &[Modifier],
    point_target: u32,
) {
    commands
        .spawn((
            widget::ui_root("Interlude"),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            GlobalZIndex(2),
            DespawnOnExit(GamePhase::Interlude),
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("Interlude Panel"),
                Node {
                    width: percent(90),
                    max_width: px(920),
                    padding: UiRect::all(px(28)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(20),
                    border_radius: BorderRadius::all(px(18)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.95)),
            ))
            .with_children(|panel| {
                panel.spawn(widget::header("Next Round"));
                panel
                    .spawn((
                        Name::new("Modifiers Row"),
                        Node {
                            width: percent(100),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::FlexStart,
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(24),
                            row_gap: px(24),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Name::new("Removed Column"),
                            Node {
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(8),
                                ..default()
                            },
                        ))
                        .with_children(|column| {
                            column.spawn((
                                widget::label("Removed"),
                                TextLayout::new_with_justify(Justify::Center),
                                TextBounds::new_horizontal(210.0),
                            ));
                            column.spawn(modifier_card_from_option(
                                removed_modifier,
                                "No modifier removed this round.",
                            ));
                        });

                        row.spawn((
                            Name::new("Active Column"),
                            Node {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                align_items: AlignItems::Center,
                                row_gap: px(8),
                                ..default()
                            },
                        ))
                        .with_children(|column| {
                            column.spawn((
                                widget::label("Active"),
                                TextLayout::new_with_justify(Justify::Center),
                                TextBounds::new_horizontal(210.0),
                            ));
                            column
                                .spawn((
                                    Name::new("Active Cards"),
                                    Node {
                                        width: percent(100),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Row,
                                        flex_wrap: FlexWrap::Wrap,
                                        column_gap: px(16),
                                        row_gap: px(16),
                                        ..default()
                                    },
                                ))
                                .with_children(|cards| {
                                    let last_index = active_modifiers.len().saturating_sub(1);
                                    if active_modifiers.is_empty() {
                                        cards.spawn(modifier_card(
                                            "None".to_string(),
                                            "No active modifiers.".to_string(),
                                        ));
                                    } else {
                                        for (index, modifier) in active_modifiers.iter().enumerate()
                                        {
                                            let is_new = index == last_index;
                                            let name = if is_new {
                                                format!("{} (new)", modifier.name())
                                            } else {
                                                modifier.name().to_string()
                                            };
                                            cards.spawn(modifier_card(
                                                name,
                                                modifier.description().to_string(),
                                            ));
                                        }
                                    }
                                });
                        });
                    });

                panel.spawn(widget::label(format!("New Target: {point_target}")));
                panel.spawn(widget::button("Start Round", start_next_round));
            });
        });
}

fn modifier_card_from_option(modifier: Option<Modifier>, empty_description: &str) -> impl Bundle {
    match modifier {
        Some(modifier) => modifier_card(
            modifier.name().to_string(),
            modifier.description().to_string(),
        ),
        None => modifier_card("None".to_string(), empty_description.to_string()),
    }
}

fn modifier_card(name: String, description: String) -> impl Bundle {
    (
        Name::new(format!("Modifier Card {name}")),
        Node {
            width: px(210),
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
                Text(name),
                TextFont::from_font_size(22.0),
                TextColor(ui_palette::HEADER_TEXT),
                TextLayout::new_with_justify(Justify::Center),
                TextBounds::new_horizontal(186.0),
            ),
            (
                Name::new("Modifier Description"),
                Text(description),
                TextFont::from_font_size(16.0),
                TextColor(ui_palette::LABEL_TEXT),
                TextLayout::new_with_justify(Justify::Center),
                TextBounds::new_horizontal(186.0),
            ),
        ],
    )
}

fn start_next_round(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<GamePhase>>) {
    next_state.set(GamePhase::Herding);
}
