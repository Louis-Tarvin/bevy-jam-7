use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Modifier {
    HyperSheep,
    MoonGravity,
    Placeholder1,
    Placeholder2,
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::HyperSheep => "Hyper Sheep",
            Modifier::MoonGravity => "Moon Gravity",
            Modifier::Placeholder1 => "Placeholder 1",
            Modifier::Placeholder2 => "Placeholder 2",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Modifier::HyperSheep => "Sheep move faster and hop higher.",
            Modifier::MoonGravity => "Lower gravity makes sheep floaty.",
            Modifier::Placeholder1 => "Placeholder modifier.",
            Modifier::Placeholder2 => "Placeholder modifier.",
        }
    }
}

impl Distribution<Modifier> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Modifier {
        match rng.random_range(0..4) {
            0 => Modifier::HyperSheep,
            1 => Modifier::MoonGravity,
            2 => Modifier::Placeholder1,
            _ => Modifier::Placeholder2,
        }
    }
}
