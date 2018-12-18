use std::collections::{BinaryHeap, HashMap};

use super::{Species, Sprite, SpriteStatus};
use crate::geometry::{BoundingBox, Direction, Point};

#[derive(Debug, Clone)]
pub struct Sprites {
    sprites: HashMap<Point, Sprite>,
}

impl Default for Sprites {
    fn default() -> Self {
        Self::new()
    }
}

impl Sprites {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    pub fn place(&mut self, position: Point, sprite: Sprite) {
        self.sprites.insert(position, sprite);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Point, &Sprite)> {
        self.sprites.iter()
    }

    pub fn sprites(&self) -> impl Iterator<Item = &Sprite> {
        self.sprites.values()
    }

    pub fn positions(&self) -> impl Iterator<Item = &Point> {
        self.sprites.keys()
    }

    pub fn get(&self, point: Point) -> Option<&Sprite> {
        self.sprites.get(&point)
    }

    pub fn peek(&self) -> Option<&Sprite> {
        let positions: BinaryHeap<Point> = self.positions().cloned().collect();
        positions.peek().and_then(|p| self.get(*p))
    }

    pub fn step(&mut self, point: Point, direction: Direction) {
        let sprite = self.sprites.remove(&point).unwrap();
        self.place(point.step(direction), sprite);
    }

    pub fn attack(&mut self, aggressor: Point, target: Point) -> SpriteStatus {
        let power = self.sprites[&aggressor].attack();
        let victim = self.sprites.get_mut(&target).unwrap();
        let result = victim.wound(power);

        // Remove corpses from the battlefield.
        if let SpriteStatus::Dead = result {
            self.sprites.remove(&target);
        };
        result
    }

    pub fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for position in self.sprites.keys() {
            bbox.include(*position);
        }
        bbox
    }

    pub fn victorious(&self) -> Option<Species> {
        self.peek().and_then(|s| {
            let sp = s.species();
            if !self.sprites().any(|s| s.species().is_enemy(sp)) {
                Some(sp)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::Species;

    #[test]
    fn sprites() {
        let mut s = Sprites::new();

        let sprite_1 = Sprite::new(Species::Elf, 200, 3);
        s.place(Point::new(1, 1), sprite_1.clone());

        let sprite_2 = Sprite::new(Species::Elf, 200, 3);
        s.place(Point::new(1, 3), sprite_2.clone());

        let mut spos = s.positions().cloned().collect::<Vec<_>>();
        spos.sort();
        assert_eq!(spos, vec![Point::new(1, 3), Point::new(1, 1)]);
    }
}
