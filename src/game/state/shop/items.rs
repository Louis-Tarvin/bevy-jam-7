use bevy::prelude::Reflect;
use rand::Rng;

use crate::game::state::GameState;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ItemType {
    Boost(Boost),
    Charm(Charm),
}

impl ItemType {
    pub fn name(&self) -> &'static str {
        match self {
            ItemType::Boost(boost) => boost.name(),
            ItemType::Charm(charm) => charm.name(),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ItemType::Boost(boost) => boost.description(),
            ItemType::Charm(charm) => charm.description(),
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            ItemType::Boost(boost) => boost.price(),
            ItemType::Charm(charm) => charm.price(),
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            ItemType::Boost(_) => "Boost",
            ItemType::Charm(_) => "Charm",
        }
    }

    pub fn random_unique(count: usize, owned_charms: &[Charm]) -> Vec<Self> {
        let mut rng = rand::rng();
        let mut items = Vec::with_capacity(count);

        let boosts = [
            Boost::BlueSheep,
            Boost::RedSheep,
            Boost::BarkPower,
            Boost::MaxCharms,
        ];
        let boost_idx = rng.random_range(0..boosts.len());
        items.push(ItemType::Boost(boosts[boost_idx]));

        let charm_pool = [
            Charm::GoldenSheep,
            Charm::HalfTimeDoubleSheep,
            Charm::ChanceBlueOnBuy,
            Charm::ChanceRedOnBuy,
            Charm::Exponential,
            // Charm::WellTrained,
            Charm::Evolution,
            Charm::Cloning,
            Charm::ShopCount,
            Charm::Ink,
            Charm::RedToGold,
        ];
        let available_charms: Vec<Charm> = charm_pool
            .into_iter()
            .filter(|charm| !owned_charms.contains(charm))
            .collect();

        while items.len() < count && items.len() - 1 < available_charms.len() {
            let charm_idx = rng.random_range(0..available_charms.len());
            let next = ItemType::Charm(available_charms[charm_idx]);
            if !items.contains(&next) {
                items.push(next);
            }
        }

        items
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum Boost {
    BlueSheep,
    RedSheep,
    BarkPower,
    MaxCharms,
}

impl Boost {
    pub fn name(&self) -> &'static str {
        match self {
            Boost::BlueSheep => "Blue Sheep",
            Boost::RedSheep => "Red Sheep",
            Boost::BarkPower => "Bark Power",
            Boost::MaxCharms => "Dream Catcher",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Boost::BlueSheep => "Apply blue wool to one of your sheep (5 points)",
            Boost::RedSheep => "Apply red wool to one of your sheep (points x1.5)",
            Boost::BarkPower => "Your bark affects sheep in a wider area.",
            Boost::MaxCharms => "1 in 4 chance to increase the maximum number of charms.",
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            Boost::BlueSheep => 2,
            Boost::RedSheep => 2,
            Boost::BarkPower => 2,
            Boost::MaxCharms => 3,
        }
    }

    pub fn apply(&self, state: &mut GameState) {
        match self {
            Boost::BlueSheep => state.blue_sheep_count += 1,
            Boost::RedSheep => state.red_sheep_count += 1,
            Boost::BarkPower => state.player_bark_radius += 1.0,
            Boost::MaxCharms => {
                let rng = &mut rand::rng();
                if rng.random_ratio(1, 4) {
                    state.max_charms += 1;
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum Charm {
    GoldenSheep,
    HalfTimeDoubleSheep,
    ChanceBlueOnBuy,
    ChanceRedOnBuy,
    Exponential,
    WellTrained,
    DoubleCountRadius,
    Evolution,
    Cloning,
    ShopCount,
    Ink,
    RedToGold,
}

impl Charm {
    pub fn name(&self) -> &'static str {
        match self {
            Charm::GoldenSheep => "Golden Sheep",
            Charm::HalfTimeDoubleSheep => "Frantic Herding",
            Charm::ChanceBlueOnBuy => "Blue Chance",
            Charm::ChanceRedOnBuy => "Red Chance",
            Charm::Exponential => "Mitosis",
            Charm::WellTrained => "Well Trained",
            Charm::DoubleCountRadius => "Eager",
            Charm::Evolution => "Evolution",
            Charm::Cloning => "Cloning",
            Charm::ShopCount => "Fully Stocked",
            Charm::Ink => "Ink",
            Charm::RedToGold => "Rose Gold",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Charm::GoldenSheep => "Spawn a golden sheep that gives 1 money when counted.",
            Charm::HalfTimeDoubleSheep => "Halve the timer but spawn double the sheep.",
            Charm::ChanceBlueOnBuy => {
                "Each time you buy a sheep it has a 1 in 4 chance to be blue."
            }
            Charm::ChanceRedOnBuy => "Each time you buy a sheep it has a 1 in 4 chance to be red.",
            Charm::Exponential => {
                "When a black sheep is counted, spawn two new black sheep at random locations."
            }
            Charm::WellTrained => "Sheep come towards you when you bark.",
            Charm::DoubleCountRadius => "Double the radius",
            Charm::Evolution => {
                "White sheep score 0, but every 5th white sheep counted becomes permanently blue."
            }
            Charm::Cloning => {
                "For the first sheep counted each round, a clone will be permanently added to your flock."
            }
            Charm::ShopCount => "The shop sells an additional item.",
            Charm::Ink => "Double the probability that a white sheep will spawn as black.",
            Charm::RedToGold => "If the first sheep to be counted is red, it turns gold.",
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            Charm::GoldenSheep => 5,
            Charm::HalfTimeDoubleSheep => 4,
            Charm::ChanceBlueOnBuy => 3,
            Charm::ChanceRedOnBuy => 3,
            Charm::Exponential => 5,
            Charm::WellTrained => 4,
            Charm::DoubleCountRadius => 3,
            Charm::Evolution => 5,
            Charm::Cloning => 4,
            Charm::ShopCount => 3,
            Charm::Ink => 3,
            Charm::RedToGold => 4,
        }
    }
}
