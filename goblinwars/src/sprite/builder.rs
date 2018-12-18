use std::collections::HashMap;

use super::{Health, Species, Sprite};

/// A stat builder, which provides default
/// stats for sprites based on the sprite's
/// species
#[derive(Debug, Clone)]
pub struct StatBuilder {
    default: Health,
    species: HashMap<Species, Health>,
}

impl StatBuilder {
    pub fn new(default: Health) -> Self {
        Self {
            default,
            species: HashMap::new(),
        }
    }

    pub fn for_species(mut self, species: Species, stat: Health) -> Self {
        self.species.insert(species, stat);
        self
    }

    fn get(&self, species: Species) -> Health {
        *self.species.get(&species).unwrap_or(&self.default)
    }
}

#[derive(Debug, Clone)]
pub struct SpriteBuilder {
    health: StatBuilder,
    attack: StatBuilder,
}

impl Default for SpriteBuilder {
    fn default() -> Self {
        Self {
            health: StatBuilder::new(200),
            attack: StatBuilder::new(3),
        }
    }
}

impl SpriteBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn with_health(mut self, species: Species, health: Health) -> Self {
        self.health = self.health.for_species(species, health);
        self
    }

    fn with_attack(mut self, species: Species, attack: Health) -> Self {
        self.attack = self.attack.for_species(species, attack);
        self
    }

    pub fn build(&self, species: Species) -> Sprite {
        Sprite::new(species, self.health.get(species), self.attack.get(species))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let b = SpriteBuilder::default();
        assert_eq!(b.build(Species::Elf), Sprite::new(Species::Elf, 200, 3));
        assert_eq!(
            b.build(Species::Goblin),
            Sprite::new(Species::Goblin, 200, 3)
        );

        let b = b
            .with_attack(Species::Elf, 6)
            .with_health(Species::Goblin, 100);
        assert_eq!(b.build(Species::Elf), Sprite::new(Species::Elf, 200, 6));
        assert_eq!(
            b.build(Species::Goblin),
            Sprite::new(Species::Goblin, 100, 3)
        );
    }
}
