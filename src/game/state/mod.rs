use bevy::prelude::*;
use rand::Rng;

use crate::{game::modifiers::Modifier, screens::Screen};

mod herding;
mod interlude;

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<GamePhase>();
    app.insert_resource(GameState::default());
    app.add_plugins((herding::plugin, interlude::plugin));
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
    pub active_modifiers: Vec<Modifier>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            sheep_count: 10,
            countdown: Timer::from_seconds(120.0, TimerMode::Once),
            points: 0,
            point_target: 10,
            active_modifiers: Vec::new(),
        }
    }
}

impl GameState {
    pub fn new_round(&mut self) -> Option<Modifier> {
        self.countdown.reset();
        self.points = 0;
        let rng = &mut rand::rng();
        let modifier: Modifier = rng.random();
        self.active_modifiers.push(modifier);
        if self.active_modifiers.len() > 3 {
            Some(self.active_modifiers.remove(0))
        } else {
            None
        }
    }

    pub fn is_modifier_active(&self, modifier: Modifier) -> bool {
        self.active_modifiers.contains(&modifier)
    }
}
