use bevy::prelude::*;

use crate::{game::level::start_music, screens::Screen};

pub mod camera;
pub mod level;
pub mod modifiers;
pub mod movement;
pub mod player;
pub mod sheep;
pub mod state;
pub mod ufo;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        movement::plugin,
        player::plugin,
        sheep::plugin,
        camera::plugin,
        state::plugin,
        ufo::plugin,
    ));
    app.add_systems(OnEnter(Screen::Gameplay), start_music);
}
