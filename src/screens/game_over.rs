use bevy::prelude::*;

use crate::{game::state::GameState, screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), spawn_game_over_screen);
}

fn spawn_game_over_screen(mut commands: Commands, game_state: Res<GameState>) {
    commands.spawn((
        widget::ui_root("Game Over Screen"),
        DespawnOnExit(Screen::GameOver),
        children![(
            widget::panel(),
            children![
                widget::header("Game Over"),
                widget::label(format!("Completed rounds: {}", game_state.completed_rounds)),
                widget::label(format!("Sheep in flock: {}", game_state.sheep_count)),
                widget::button("Main Menu", return_to_main_menu),
            ],
        )],
    ));
}

fn return_to_main_menu(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
