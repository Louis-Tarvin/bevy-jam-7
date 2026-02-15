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
    Vignette,
    Night,
    SheepSphere,
    DogSphere,
    FeverDream,
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::HyperSheep => "Hyper Sheep",
            Modifier::MoonGravity => "Moon Gravity",
            Modifier::Ufo => "UFO",
            Modifier::Space => "Space",
            Modifier::TeleportingBark => "Teleporting Bark",
            Modifier::Vignette => "Brain Fog",
            Modifier::Night => "Night time",
            Modifier::SheepSphere => "Rollin'",
            Modifier::DogSphere => "Spherical",
            Modifier::FeverDream => "Feverdream",
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
            Modifier::Vignette => {
                "The clouds around the edges of the screen grow bigger, restricting your visibility."
            }
            Modifier::Night => "The sheep start asleep.",
            Modifier::SheepSphere => "Sheep roll around like a ball.",
            Modifier::DogSphere => "You roll around like a ball.",
            Modifier::FeverDream => "Increases the intensity of certain other active modifiers",
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
            Modifier::Vignette => Hard,
            Modifier::Night => Medium,
            Modifier::SheepSphere => Medium,
            Modifier::DogSphere => Easy,
            Modifier::FeverDream => Hard,
        }
    }
}

impl Distribution<Modifier> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Modifier {
        let pool = vec![
            Modifier::HyperSheep,
            Modifier::MoonGravity,
            Modifier::Ufo,
            Modifier::Space,
            Modifier::TeleportingBark,
            Modifier::Vignette,
            Modifier::Night,
            Modifier::SheepSphere,
            Modifier::DogSphere,
            Modifier::FeverDream,
        ];
        pool[rng.random_range(0..pool.len())]
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
