use bevy::{math::ops::floor, prelude::*};

use crate::{
    game::{
        modifiers::Modifier,
        state::{
            GamePhase, GameState,
            shop::{
                ShopOffers,
                items::{Charm, ItemType},
            },
        },
    },
    theme::prelude::*,
};

#[derive(Component)]
pub struct ShopUiRoot;

pub fn draw_shop_ui(mut commands: Commands, game_state: &GameState, shop_offers: &ShopOffers) {
    let active_modifiers = game_state.active_modifiers.clone();
    let charms = game_state.charms.clone();
    let max_charms = game_state.max_charms;
    let money = game_state.money;
    let point_target = game_state.point_target;
    let offers = shop_offers.items.clone();
    let charms_full = game_state.charms_full();
    commands.spawn((
        ShopUiRoot,
        widget::ui_root("Shop UI"),
        DespawnOnExit(GamePhase::Shop),
        children![(
            widget::panel(),
            children![(
                widget::columns(),
                children![
                    (
                        Name::new("Left Column"),
                        Node {
                            min_width: px(250),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(12),
                            ..default()
                        },
                        children![
                            widget::header("Modifiers"),
                            (
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8),
                                    ..default()
                                },
                                Children::spawn(SpawnIter(
                                    active_modifiers.into_iter().map(modifier_card)
                                ))
                            ),
                        ]
                    ),
                    (
                        Name::new("Center Column"),
                        Node {
                            min_width: px(450),
                            flex_grow: 2.0,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(12),
                            ..default()
                        },
                        children![
                            widget::header("Shop"),
                            (
                                widget::row(),
                                children![
                                    widget::label(format!("Money: {}", money)),
                                    widget::button_medium("Reroll (1)", draw_new_items),
                                ]
                            ),
                            (
                                Node {
                                    width: percent(100),
                                    justify_content: JustifyContent::SpaceAround,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::Wrap,
                                    column_gap: px(10),
                                    row_gap: px(10),
                                    ..default()
                                },
                                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                                    for (slot, item) in offers.into_iter().enumerate() {
                                        match item {
                                            Some(item) => {
                                                parent.spawn(item_card(
                                                    slot,
                                                    item,
                                                    money,
                                                    charms_full,
                                                ));
                                            }
                                            None => {
                                                parent.spawn(bought_item_card());
                                            }
                                        }
                                    }
                                })),
                            ),
                            widget::divider(),
                            (
                                widget::row(),
                                children![
                                    widget::label(format!(
                                        "Total Sheep: {}",
                                        game_state.sheep_count
                                    )),
                                    widget::button_medium("Buy a Sheep (1)", buy_sheep)
                                ]
                            ),
                            widget::divider(),
                            widget::label(format!("Points target: {}", point_target)),
                            widget::button("Start", start_next_round)
                        ]
                    ),
                    (
                        Name::new("Right Column"),
                        Node {
                            min_width: px(250),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                            ..default()
                        },
                        children![
                            widget::label(format!("Charms ({}/{})", charms.len(), max_charms)),
                            (
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8),
                                    ..default()
                                },
                                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                                    if charms.is_empty() {
                                        parent.spawn(widget::label("No charms equipped"));
                                        return;
                                    }

                                    for (slot, charm) in charms.into_iter().enumerate() {
                                        parent.spawn(charm_card(slot, charm));
                                    }
                                })),
                            ),
                        ]
                    ),
                ]
            )]
        )],
    ));
}

fn charm_card(slot: usize, charm: Charm) -> impl Bundle {
    let sell_price = floor(charm.price() as f32 / 2.0);

    (
        Name::new(format!("Charm Card {}", charm.name())),
        Node {
            width: px(250),
            max_width: percent(100),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.16, 0.2, 0.18, 0.95)),
        children![
            (
                Name::new("Charm Name"),
                Text(charm.name().to_string()),
                TextFont::from_font_size(22.0),
                TextColor(ui_palette::HEADER_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ),
            (
                Name::new("Charm Description"),
                Text(charm.description().to_string()),
                TextFont::from_font_size(16.0),
                TextColor(ui_palette::LABEL_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ),
            widget::button_medium(
                format!("Sell (+{})", sell_price),
                move |_: On<Pointer<Click>>, mut game_state: ResMut<GameState>| {
                    sell_charm(slot, &mut game_state);
                },
            ),
        ],
    )
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

fn draw_new_items(
    _: On<Pointer<Click>>,
    mut game_state: ResMut<GameState>,
    mut shop_offers: ResMut<ShopOffers>,
) {
    if game_state.money == 0 {
        return;
    }
    game_state.money -= 1;
    shop_offers.reroll();
}

fn item_card(slot: usize, item: ItemType, money: u32, charms_full: bool) -> impl Bundle {
    let price = item.price();
    let buy_text = format!("Buy ({})", price);

    (
        Name::new(format!("Shop Item Card {}", item.name())),
        Node {
            width: px(250),
            max_width: percent(100),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.95)),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Name::new("Item Type"),
                Text(item.kind_label().to_string()),
                TextFont::from_font_size(14.0),
                TextColor(ui_palette::LABEL_TEXT),
            ));
            parent.spawn((
                Name::new("Item Name"),
                Text(item.name().to_string()),
                TextFont::from_font_size(22.0),
                TextColor(ui_palette::HEADER_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ));
            parent.spawn((
                Name::new("Item Description"),
                Text(item.description().to_string()),
                TextFont::from_font_size(16.0),
                TextColor(ui_palette::LABEL_TEXT),
                TextLayout::new_with_justify(Justify::Center),
            ));

            if money >= price && !(matches!(item, ItemType::Charm(_)) && charms_full) {
                parent.spawn(widget::button_medium(
                    buy_text.clone(),
                    move |_: On<Pointer<Click>>,
                          mut game_state: ResMut<GameState>,
                          mut shop_offers: ResMut<ShopOffers>| {
                        buy_shop_item(slot, &mut game_state, &mut shop_offers);
                    },
                ));
            } else {
                parent.spawn(widget::button_medium_disabled(buy_text.clone()));
            }
        })),
    )
}

fn bought_item_card() -> impl Bundle {
    (
        Name::new("Shop Item Card Bought"),
        Node {
            width: px(250),
            max_width: percent(100),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.95)),
        children![(
            Name::new("Bought Label"),
            Text("Bought".to_string()),
            TextFont::from_font_size(28.0),
            TextColor(ui_palette::LABEL_TEXT.with_alpha(0.75)),
            TextLayout::new_with_justify(Justify::Center),
        )],
    )
}

fn buy_sheep(_: On<Pointer<Click>>, mut game_state: ResMut<GameState>) {
    if game_state.money < 1 {
        return;
    }
    game_state.sheep_count += 1;
    game_state.money -= 1;
}

fn buy_shop_item(slot: usize, game_state: &mut GameState, shop_offers: &mut ShopOffers) {
    let Some(Some(item)) = shop_offers.items.get(slot).copied() else {
        return;
    };
    if game_state.money < item.price() {
        return;
    }

    match item {
        ItemType::Boost(boost) => boost.apply(game_state),
        ItemType::Charm(charm) => {
            if game_state.charms_full() {
                return;
            }
            game_state.charms.push(charm);
        }
    }

    game_state.money -= item.price();
    shop_offers.items[slot] = None;
}

fn sell_charm(slot: usize, game_state: &mut GameState) {
    let Some(charm) = game_state.charms.get(slot).copied() else {
        return;
    };

    game_state.charms.remove(slot);
    game_state.money += charm.price();
}

pub fn redraw_shop_ui(
    mut commands: Commands,
    game_state: Res<GameState>,
    shop_offers: Res<ShopOffers>,
    roots: Query<Entity, With<ShopUiRoot>>,
) {
    if !game_state.is_changed() && !shop_offers.is_changed() {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }
    draw_shop_ui(commands, &game_state, &shop_offers);
}
