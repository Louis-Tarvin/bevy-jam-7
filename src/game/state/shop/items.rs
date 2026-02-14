use bevy::prelude::Reflect;
use rand::Rng;

use crate::game::state::GameState;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ItemType {
    Boost(Boost),
    Charm(Charm),
}

impl ItemType {
    pub const SHOP_OFFER_COUNT: usize = 3;

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

    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let pool = [
            ItemType::Boost(Boost::BlueSheep),
            ItemType::Boost(Boost::RedSheep),
            ItemType::Boost(Boost::BarkPower),
            ItemType::Boost(Boost::MaxCharms),
            ItemType::Charm(Charm::GoldenSheep),
            ItemType::Charm(Charm::HalfTimeDoubleSheep),
            ItemType::Charm(Charm::ChanceBlueOnBuy),
            ItemType::Charm(Charm::ChanceRedOnBuy),
            ItemType::Charm(Charm::Exponential),
            ItemType::Charm(Charm::WellTrained),
        ];
        let idx = rng.random_range(0..pool.len());
        pool[idx]
    }

    pub fn random_unique(count: usize, owned_charms: &[Charm]) -> Vec<Self> {
        let mut rng = rand::rng();
        let mut items = Vec::with_capacity(count);
        let mut attempts = 0;
        while items.len() < count && attempts < 100 {
            let next = Self::random(&mut rng);
            if matches!(next, ItemType::Charm(charm) if owned_charms.contains(&charm))
                || items.contains(&next)
            {
                attempts += 1;
                continue;
            }
            items.push(next);
            attempts += 1;
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
            Boost::BarkPower => 3,
            Boost::MaxCharms => 3,
        }
    }

    pub fn apply(&self, state: &mut GameState) {
        match self {
            Boost::BlueSheep => state.blue_sheep_count += 1,
            Boost::RedSheep => state.red_sheep_count += 1,
            Boost::BarkPower => state.player_bark_radius += 2.0,
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
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Charm::GoldenSheep => "Spawn a golden sheep that gives 1 money when counted.",
            Charm::HalfTimeDoubleSheep => "Halve the timer but spawn double the sheep.",
            Charm::ChanceBlueOnBuy => "Each time you buy a sheep it has a 10% chance to be blue.",
            Charm::ChanceRedOnBuy => "Each time you buy a sheep it has a 10% chance to be red.",
            Charm::Exponential => {
                "When a black sheep is counted, spawn two new black sheep at random locations."
            }
            Charm::WellTrained => "Sheep come towards you when you bark.",
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
        }
    }
}
