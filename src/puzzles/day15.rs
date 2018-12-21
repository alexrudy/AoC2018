use std::error::Error;

use goblinwars::map::MapBuilder;
use goblinwars::sprite::{Species, SpriteBuilder};
use goblinwars::{Game, GameOutcome};

type Result<T> = ::std::result::Result<T, Box<Error>>;

pub(crate) fn main() -> Result<()> {
    use crate::input_to_string;

    let map = input_to_string(15)?;

    let mut game = Game::new(
        MapBuilder::default()
            .build(&map)
            .map_err(|e| e.to_string())?,
    );

    let outcome = game
        .run(|_, t| {
            eprint!("\r Time: {}", t);
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    eprint!("\n");
    if let GameOutcome::Complete(stats) = outcome {
        println!("Part 1: {:3}", stats.score);
    }

    for attack in 4.. {
        let builder = MapBuilder::new(SpriteBuilder::default().with_attack(Species::Elf, attack));

        let mut game = Game::new(builder.build(&map).map_err(|e| e.to_string())?);

        let n_elves = game.map().alive(Species::Elf);

        let outcome = game
            .run(|_, t| {
                eprint!("\r Time: {:3} ({:3})", t, attack);
                Ok(())
            })
            .map_err(|e| e.to_string())?;

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
mod tests {

    use super::*;

    use test::Bencher;

    #[bench]
    fn bench_pathfinder_nocache(b: &mut Bencher) {
        use crate::input_to_string;

        let map = MapBuilder::default()
            .build(&input_to_string(15).unwrap())
            .map_err(|e| e.to_string())
            .unwrap();

        // b.iter(|| {
        //     map.clone().round().play();
        // });
    }

}
