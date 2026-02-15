//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};

use crate::theme::{interaction::InteractionPalette, palette::*};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
        TextColor(HEADER_TEXT),
    )
}

pub fn column_header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(30.0),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(24.0),
        TextColor(LABEL_TEXT),
    )
}

pub fn panel() -> impl Bundle {
    (
        Name::new("Panel"),
        Node {
            width: percent(95),
            min_width: px(1080),
            max_width: px(1500),
            padding: UiRect::all(px(28)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            border_radius: BorderRadius::all(px(18)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.95)),
    )
}

pub fn columns() -> impl Bundle {
    (
        Name::new("Columns"),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(10),
            width: percent(100),
            height: percent(100),
            ..default()
        },
    )
}

pub fn row() -> impl Bundle {
    (
        Name::new("Row"),
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
    )
}

/// A horizontal divider line.
pub fn divider() -> impl Bundle {
    (
        Name::new("Divider"),
        Node {
            width: percent(90),
            height: px(2),
            ..default()
        },
        BackgroundColor(LABEL_TEXT.with_alpha(0.35)),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        40.0,
        action,
        Node {
            width: px(380),
            height: px(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
    )
}

pub fn button_medium<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        20.0,
        action,
        Node {
            width: px(150),
            height: px(40),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
    )
}

pub fn button_medium_disabled(text: impl Into<String>) -> impl Bundle {
    button_disabled(
        text,
        20.0,
        Node {
            width: px(150),
            height: px(40),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        40.0,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    font_size: f32,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(font_size),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

/// A disabled button with text and no interaction.
fn button_disabled(
    text: impl Into<String>,
    font_size: f32,
    button_bundle: impl Bundle,
) -> impl Bundle {
    (
        Name::new("Button"),
        Node::default(),
        children![(
            Name::new("Button Inner"),
            BackgroundColor(BUTTON_BACKGROUND.with_alpha(0.4)),
            button_bundle,
            Pickable::IGNORE,
            children![(
                Name::new("Button Text"),
                Text(text.into()),
                TextFont::from_font_size(font_size),
                TextColor(BUTTON_TEXT.with_alpha(0.55)),
                Pickable::IGNORE,
            )],
        )],
    )
}
