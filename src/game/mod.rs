use bevy::prelude::*;

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
}
