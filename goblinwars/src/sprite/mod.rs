#![allow(dead_code)]

use std::fmt;
use std::str::FromStr;

mod builder;
mod collection;

pub use self::builder::SpriteBuilder;
pub use self::collection::Sprites;

pub type Health = u32;

#[derive(Debug, Clone, PartialEq, Eq, Fail)]
pub enum ParseSpeciesError {
    #[fail(display = "No characters to parse")]
    NoCharacters,

    #[fail(display = "Too many characters to parse: {}", _0)]
    TooManyCharacters(String),

    #[fail(display = "Unknown species: {}", _0)]
    UnknownSpecies(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Species {
    Elf,
    Goblin,
}

impl Species {
    pub fn is_enemy(self, other: Species) -> bool {
        !self.eq(&other)
    }

    pub fn plural(self) -> &'static str {
        match self {
            Species::Elf => "Elves",
            Species::Goblin => "Goblins",
        }
    }
}

impl fmt::Display for Species {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Species::Elf => write!(f, "E"),
            Species::Goblin => write!(f, "G"),
        }
    }
}

impl FromStr for Species {
    type Err = ParseSpeciesError;

    fn from_str(s: &str) -> Result<Self, ParseSpeciesError> {
        if s.is_empty() {
            return Err(ParseSpeciesError::NoCharacters);
        }
        if s.len() != 1 {
            return Err(ParseSpeciesError::TooManyCharacters(s.to_string()));
        }

        match s {
            "E" => Ok(Species::Elf),
            "G" => Ok(Species::Goblin),
            _ => Err(ParseSpeciesError::UnknownSpecies(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteStatus {
    Alive(Health),
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    species: Species,
    hit_points: Health,
    attack_power: Health,
}

impl Sprite {
    pub fn new(species: Species, health: Health, attack: Health) -> Self {
        Self {
            species,
            hit_points: health,
            attack_power: attack,
        }
    }

    pub fn attack(&self) -> Health {
        self.attack_power
    }

    pub fn wound(&mut self, attack: Health) -> SpriteStatus {
        self.hit_points = self.hit_points.saturating_sub(attack);
        self.status()
    }

    pub fn status(&self) -> SpriteStatus {
        match self.hit_points {
            0 => SpriteStatus::Dead,
            h => SpriteStatus::Alive(h),
        }
    }

    pub fn health(&self) -> Health {
        self.hit_points
    }

    pub fn is_enemy(&self, other: &Self) -> bool {
        self.species.is_enemy(other.species)
    }

    pub fn species(&self) -> Species {
        self.species
    }

    pub fn info(&self) -> SpriteInfo {
        SpriteInfo { sprite: self }
    }

    pub fn glyph(&self) -> SpriteGlyph {
        SpriteGlyph { sprite: self }
    }

    pub fn with_health(mut self, health: Health) -> Self {
        self.hit_points = health;
        self
    }

    pub fn with_attack(mut self, attack: Health) -> Self {
        self.attack_power = attack;
        self
    }
}

pub struct SpriteInfo<'s> {
    sprite: &'s Sprite,
}

impl<'s> fmt::Display for SpriteInfo<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.sprite.species, self.sprite.hit_points)
    }
}

pub struct SpriteGlyph<'s> {
    sprite: &'s Sprite,
}

impl<'s> fmt::Display for SpriteGlyph<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.sprite.species)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species() {
        assert_eq!("G".parse::<Species>(), Ok(Species::Goblin));
        assert_eq!("E".parse::<Species>(), Ok(Species::Elf));
        assert_eq!("".parse::<Species>(), Err(ParseSpeciesError::NoCharacters));
        assert_eq!(
            "B".parse::<Species>(),
            Err(ParseSpeciesError::UnknownSpecies("B".to_string()))
        );
        assert_eq!(
            "Blah".parse::<Species>(),
            Err(ParseSpeciesError::TooManyCharacters("Blah".to_string()))
        );

        assert_eq!(&format!("{}", Species::Elf), "E");
        assert_eq!(&format!("{}", Species::Goblin), "G");

        assert!(Species::Elf.is_enemy(Species::Goblin));
        assert!(Species::Goblin.is_enemy(Species::Elf));
        assert!(!Species::Goblin.is_enemy(Species::Goblin));
        assert!(!Species::Elf.is_enemy(Species::Elf));
    }

    #[test]
    fn sprite_display() {
        let elf = Sprite::new(Species::Elf, 200, 3);
        let goblin = Sprite::new(Species::Goblin, 10, 3);

        assert_eq!(&format!("{}", elf.glyph()), "E");
        assert_eq!(&format!("{}", goblin.glyph()), "G");

        assert_eq!(&format!("{}", goblin.info()), "G(10)");
        assert_eq!(&format!("{}", elf.info()), "E(200)");
    }

    #[test]
    fn sprite_attack() {
        let elf = Sprite::new(Species::Elf, 200, 3);
        let mut goblin = Sprite::new(Species::Goblin, 5, 3);

        assert!(elf.is_enemy(&goblin));
        assert!(!elf.is_enemy(&elf));

        assert_eq!(goblin.status(), SpriteStatus::Alive(5));
        assert_eq!(goblin.health(), 5);
        assert_eq!(goblin.wound(elf.attack()), SpriteStatus::Alive(2));
        assert_eq!(goblin.health(), 2);

        assert_eq!(goblin.wound(elf.attack()), SpriteStatus::Dead);
        assert_eq!(goblin.health(), 0);
        assert_eq!(goblin.status(), SpriteStatus::Dead);
    }

}
