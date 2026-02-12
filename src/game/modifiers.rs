use bevy::prelude::*;
use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum Modifier {
    HyperSheep,
    MoonGravity,
    Ufo,
    Space,
    TeleportingBark,
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::HyperSheep => "Hyper Sheep",
            Modifier::MoonGravity => "Moon Gravity",
            Modifier::Ufo => "UFO",
            Modifier::Space => "Space",
            Modifier::TeleportingBark => "Teleporting Bark",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Modifier::HyperSheep => "Sheep move faster and hop higher.",
            Modifier::MoonGravity => "Lower gravity makes sheep floaty.",
            Modifier::Ufo => "A UFO will fly around and occasionally abduct a sheep.",
            Modifier::Space => "You fly around with floaty, frictionless movement.",
            Modifier::TeleportingBark => {
                "Every time you bark you'll be teleported to a random location."
            }
        }
    }

    pub fn difficulty(&self) -> ModifierDifficulty {
        use ModifierDifficulty::*;
        match self {
            Modifier::HyperSheep => Easy,
            Modifier::MoonGravity => Medium,
            Modifier::Ufo => Hard,
            Modifier::Space => Hard,
            Modifier::TeleportingBark => Hard,
        }
    }
}

impl Distribution<Modifier> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Modifier {
        match rng.random_range(0..4) {
            0 => Modifier::HyperSheep,
            1 => Modifier::MoonGravity,
            2 => Modifier::Ufo,
            _ => Modifier::Space,
        }
    }
}

pub enum ModifierDifficulty {
    Easy,
    Medium,
    Hard,
}

impl ModifierDifficulty {
    pub fn coins_given(&self) -> u8 {
        match self {
            ModifierDifficulty::Easy => 4,
            ModifierDifficulty::Medium => 5,
            ModifierDifficulty::Hard => 6,
        }
    }
}
