use bevy::prelude::*;

pub mod camera;
pub mod level;
pub mod movement;
pub mod player;
pub mod sheep;
pub mod state;
pub mod ui;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        movement::plugin,
        player::plugin,
        sheep::plugin,
        camera::plugin,
        state::plugin,
    ));
}
