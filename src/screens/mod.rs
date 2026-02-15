//! The game's main screen states and transitions between them.

mod game_over;
mod gameplay;
mod how_to_play;
mod loading;
mod splash;
mod title;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        gameplay::plugin,
        game_over::plugin,
        loading::plugin,
        splash::plugin,
        title::plugin,
        how_to_play::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Loading,
    Gameplay,
    GameOver,
    HowToPlay,
}
