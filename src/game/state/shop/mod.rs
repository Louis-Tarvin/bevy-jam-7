use bevy::prelude::*;

use crate::game::state::{
    GamePhase,
    shop::{items::ItemType, ui::redraw_shop_ui},
};

pub mod items;
mod ui;

#[derive(Debug, Resource, Default)]
pub struct ShopOffers {
    pub items: Vec<Option<ItemType>>,
}

impl ShopOffers {
    pub fn reroll(&mut self) {
        self.items = ItemType::random_unique(ItemType::SHOP_OFFER_COUNT)
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

fn on_shop(mut shop_offers: ResMut<ShopOffers>) {
    shop_offers.reroll();
}
