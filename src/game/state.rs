use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        sheep::{SheepAssets, spawn_sheep},
        ui::{draw_herding_ui, update_herding_ui},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<GamePhase>();
    app.insert_resource(GameState::default());
    app.add_systems(OnEnter(GamePhase::Herding), on_herding);
    app.add_systems(
        Update,
        tick_countdown
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        update_herding_ui.run_if(in_state(GamePhase::Herding)),
    );
}

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(Screen = Screen::Gameplay)]
pub enum GamePhase {
    #[default]
    Herding,
    Interlude,
}

#[derive(Debug, Resource)]
pub struct GameState {
    pub sheep_count: u16,
    pub countdown: Timer,
    pub points: u32,
    pub point_target: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            sheep_count: 10,
            countdown: Timer::from_seconds(120.0, TimerMode::Once),
            points: 0,
            point_target: 10,
        }
    }
}

impl GameState {
    pub fn new_round(&mut self) {
        self.countdown.reset();
        self.points = 0;
    }
}

fn tick_countdown(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    state.countdown.tick(time.delta());
    if state.countdown.just_finished() {
        next_state.set(Screen::Title);
    }
}

fn on_herding(
    mut commands: Commands,
    sheep_assets: Res<SheepAssets>,
    mut game_state: ResMut<GameState>,
) {
    game_state.new_round();

    let count = game_state.sheep_count as usize;
    if count == 0 {
        return;
    }

    let grid = (count as f32).sqrt().ceil() as usize;
    let spacing = 10.0;
    let offset = (grid as f32 - 1.0) * 0.5;

    for i in 0..count {
        let x = (i % grid) as f32;
        let z = (i / grid) as f32;
        let pos = Vec3::new((x - offset) * spacing, 0.0, (z - offset) * spacing);
        commands.spawn(spawn_sheep(&sheep_assets, pos));
    }

    draw_herding_ui(&mut commands);
}
