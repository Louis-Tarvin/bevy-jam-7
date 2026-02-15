use bevy::prelude::*;

use crate::{screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::HowToPlay), spawn_how_to_play_screen);
}

fn spawn_how_to_play_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Screen"),
        DespawnOnExit(Screen::HowToPlay),
        children![(
            widget::panel(),
            children![
                widget::header("How to Play"),
                widget::label(r#"
The objective is to score the target number of points each round before the time runs out.

You score points by herding the sheep towards the opening, where they will be counted. Different colors of sheep score different numbers of points:
- White sheep score 1 point
- Blue sheep score 5 points
- Red sheep multiply your current point count by 1.5
- Black sheep are the same as white by default (but certain charms give them special effects)
- Gold sheep give money

Sheep run away from you when you get too close. You can also press E or SPACE to bark, which will cause sheep to flee. Sheep also gravitate towards other sheep (you can use this flocking behavior to your advantage).

At the end of a round you'll pick a 'dream modifier', which can have all kinds of crazy effects. These each last for 3 rounds.

You'll then visit a shop where you can spend money on upgrades. 'Boosts' are permanent stat boosts that you can purchase many times. 'Charms' are powerful gameplay-altering items, but you can only have 4 at a time.
"#),
                widget::button("Main Menu", return_to_main_menu),
            ],
        )],
    ));
}

fn return_to_main_menu(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
