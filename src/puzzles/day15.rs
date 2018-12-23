use failure::Error;

use goblinwars::map::MapBuilder;
use goblinwars::sprite::{Species, SpriteBuilder};
use goblinwars::{Game, GameOutcome};

type Result<T> = ::std::result::Result<T, Error>;

pub(crate) fn main() -> Result<()> {
    use crate::input_to_string;

    let map = input_to_string(15)?;

    let mut game = Game::new(MapBuilder::default().build(&map)?);

    let outcome = game.run(|_, t| {
        eprint!("\r Time: {}", t);
        Ok(())
    })?;
    eprint!("\n");
    if let GameOutcome::Complete(stats) = outcome {
        println!("Part 1: {:3}", stats.score);
    }

    for attack in 4.. {
        let builder = MapBuilder::new(SpriteBuilder::default().with_attack(Species::Elf, attack));

        let mut game = Game::new(builder.build(&map)?);

        let n_elves = game.map().alive(Species::Elf);

        let outcome = game.run(|_, t| {
            eprint!("\r Time: {:3} ({:2})", t, attack);
            Ok(())
        })?;

        if n_elves == game.map().alive(Species::Elf) {
            if let GameOutcome::Complete(stats) = outcome {
                if stats.victors == Species::Elf {
                    eprint!("\n");
                    println!("Part 2: {}", stats.score);
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
