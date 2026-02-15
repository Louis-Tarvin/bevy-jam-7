use bevy::prelude::*;

use crate::{
    audio::BgmConfig,
    game::state::{
        GamePhase, GameState,
        shop::{
            items::{Charm, ItemType},
            ui::redraw_shop_ui,
        },
    },
};

pub mod items;
mod ui;

#[derive(Debug, Resource, Default)]
pub struct ShopOffers {
    pub items: Vec<Option<ItemType>>,
}

impl ShopOffers {
    pub fn reroll(&mut self, owned_charms: &[items::Charm], count: usize) {
        self.items = ItemType::random_unique(count, owned_charms)
            .into_iter()
            .map(Some)
            .collect();
    }
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(ShopOffers::default());
    app.add_systems(OnEnter(GamePhase::Shop), on_shop);
    app.add_systems(Update, redraw_shop_ui.run_if(in_state(GamePhase::Shop)));
}

fn on_shop(
    mut shop_offers: ResMut<ShopOffers>,
    mut bgm_config: ResMut<BgmConfig>,
    game_state: Res<GameState>,
) {
    bgm_config.base_enabled = true;
    bgm_config.extra_enabled = true;
    bgm_config.percussion_enabled = false;
    let count = if game_state.is_charm_active(Charm::ShopCount) {
        4
    } else {
        3
    };
    shop_offers.reroll(&game_state.charms, count);
}
