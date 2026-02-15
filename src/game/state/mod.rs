use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    game::{modifiers::Modifier, state::shop::items::Charm},
    screens::Screen,
};

mod herding;
mod modifier_choice;
pub mod shop;

const TIMER_SECONDS: f32 = 70.0;

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<GamePhase>();
    app.insert_resource(GameState::default());
    app.insert_resource(RoundStats::default());
    app.add_plugins((herding::plugin, modifier_choice::plugin, shop::plugin));
    app.add_systems(OnEnter(Screen::Title), reset_run_state);
}

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(Screen = Screen::Gameplay)]
pub enum GamePhase {
    #[default]
    Herding,
    ModifierChoice,
    Shop,
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameState {
    pub completed_rounds: u32,
    pub sheep_count: u16,
    pub blue_sheep_count: u16,
    pub red_sheep_count: u16,
    pub black_sheep_count: u16,
    pub gold_sheep_count: u16,
    pub countdown: Timer,
    pub points: u32,
    pub point_target: u32,
    pub active_modifiers: Vec<Modifier>,
    pub money: u32,
    pub charms: Vec<Charm>,
    pub max_charms: u8,
    pub player_bark_radius: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            completed_rounds: 0,
            sheep_count: 10,
            blue_sheep_count: 1,
            red_sheep_count: 1,
            countdown: Timer::from_seconds(TIMER_SECONDS, TimerMode::Once),
            points: 0,
            point_target: 3,
            active_modifiers: Vec::new(),
            money: 0,
            charms: Vec::with_capacity(4),
            max_charms: 4,
            player_bark_radius: 12.0,
            black_sheep_count: 0,
            gold_sheep_count: 0,
        }
    }
}

impl GameState {
    pub fn new_round(&mut self) -> NewRoundInfo {
        self.completed_rounds += 1;
        self.points = 0;
        self.point_target += 2 + (self.point_target / 10);
        let removed_modifier = if self.active_modifiers.len() > 2 {
            Some(self.active_modifiers.remove(0))
        } else {
            None
        };
        let modifier_choices = self.pick_random_modifiers(2);
        NewRoundInfo {
            removed_modifier,
            modifier_choices,
        }
    }

    pub fn reset_timer(&mut self) {
        if self.is_charm_active(Charm::HalfTimeDoubleSheep) {
            self.countdown
                .set_duration(Duration::from_secs_f32(TIMER_SECONDS - 20.0));
        } else {
            self.countdown
                .set_duration(Duration::from_secs_f32(TIMER_SECONDS));
        }
        self.countdown.reset();
    }

    pub fn is_modifier_active(&self, modifier: Modifier) -> bool {
        self.active_modifiers.contains(&modifier)
    }

    pub fn is_charm_active(&self, charm: Charm) -> bool {
        self.charms.contains(&charm)
    }

    pub fn charms_full(&self) -> bool {
        self.charms.len() >= self.max_charms as usize
    }

    fn pick_random_modifiers(&self, count: usize) -> Vec<Modifier> {
        let mut choices = Vec::with_capacity(count);
        let rng = &mut rand::rng();
        let mut attempts = 0;
        while choices.len() < count && attempts < 100 {
            let modifier: Modifier = rng.random();
            if self.active_modifiers.contains(&modifier) || choices.contains(&modifier) {
                attempts += 1;
                continue;
            }
            choices.push(modifier);
            attempts += 1;
        }
        choices
    }
}

pub struct NewRoundInfo {
    removed_modifier: Option<Modifier>,
    modifier_choices: Vec<Modifier>,
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct RoundStats {
    pub sheep_counted: u16,
    pub white_sheep_counted: u16,
    pub black_sheep_counted: u16,
}

fn reset_run_state(
    mut game_state: ResMut<GameState>,
    mut round_stats: ResMut<RoundStats>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    *game_state = GameState::default();
    *round_stats = RoundStats::default();
    next_phase.set(GamePhase::Herding);
}
